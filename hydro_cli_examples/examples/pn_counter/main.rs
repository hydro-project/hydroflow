use std::collections::HashMap;
use std::time::Duration;

use futures::SinkExt;
use hydroflow::bytes::Bytes;
use hydroflow::hydroflow_syntax;
use hydroflow::serde::{Deserialize, Serialize};
use hydroflow::tokio;
use hydroflow::util::cli::ConnectedDemux;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IncrementRequest {
    tweet_id: u64,
    likes: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum GossipOrIncrement {
    Gossip(HashMap<u64, (Vec<u32>, Vec<u32>)>),
    Increment(u64, i32),
}

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let my_id: Vec<usize> = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let my_id = my_id[0];
    let num_replicas: Vec<usize> = serde_json::from_str(&std::env::args().nth(2).unwrap()).unwrap();
    let num_replicas = num_replicas[0];

    let increment_requests = ports
        .remove("increment_requests")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let query_responses = ports
        .remove("query_responses")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let to_peer = ports
        .remove("to_peer")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let from_peer = ports
        .remove("from_peer")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let mut memory_report = ports
        .remove("memory_report")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let f1 = async move {
        #[cfg(target_os = "linux")]
        loop {
            let x = procinfo::pid::stat_self().unwrap();
            let bytes = x.rss * 1024 * 4;
            memory_report
                .send(Bytes::from(serde_json::to_string(&bytes).unwrap()))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    let df = hydroflow_syntax! {
        next_state = merge()
            -> fold::<'static>((0, HashMap::<u64, (Vec<u32>, Vec<u32>)>::new()), |(_, mut cur_state), goi| {
                match goi {
                    GossipOrIncrement::Gossip(gossip) => {
                        for (counter_id, (pos, neg)) in gossip.iter() {
                            let cur_value = cur_state.entry(*counter_id).or_insert((
                                vec![0; num_replicas], vec![0; num_replicas]
                            ));

                            for i in 0..num_replicas {
                                cur_value.0[i] = std::cmp::max(cur_value.0[i], pos[i]);
                                cur_value.1[i] = std::cmp::max(cur_value.1[i], neg[i]);
                            }
                        }
                    }
                    GossipOrIncrement::Increment(counter_id, delta) => {
                        let cur_value = cur_state.entry(counter_id).or_insert((
                            vec![0; num_replicas], vec![0; num_replicas]
                        ));
                        if delta > 0 {
                            cur_value.0[my_id] += delta as u32;
                        } else {
                            cur_value.1[my_id] += (-delta) as u32;
                        }
                    }
                }

                (context.current_tick(), cur_state)
            }) -> filter(|t| t.0 == context.current_tick())
            -> map(|t| t.1) -> tee();

        source_stream(from_peer)
            -> map(|x| deserialize_from_bytes::<GossipOrIncrement>(&x.unwrap()).unwrap())
            -> next_state;

        source_stream(increment_requests)
            -> map(|x| String::from_utf8(x.unwrap().to_vec()).unwrap())
            -> map(|x| serde_json::from_str::<IncrementRequest>(&x).unwrap())
            -> map(|t| GossipOrIncrement::Increment(t.tweet_id, t.likes))
            -> next_state;

        all_peers = source_iter(0..num_replicas)
            -> filter(|x| *x != my_id);

        all_peers -> [0] broadcaster;
        next_state -> [1] broadcaster;
        broadcaster = cross_join::<'static, 'tick>()
            -> map(|(peer, state)| {
                (peer as u32, serialize_to_bytes(GossipOrIncrement::Gossip(state)))
            })
            -> dest_sink(to_peer);

        next_state
            -> map(|a: HashMap<u64, (Vec<u32>, Vec<u32>)>| {
                a.into_iter().map(|(k, (pos, neg))| {
                    (k, pos.iter().sum::<u32>() as i32 - neg.iter().sum::<u32>() as i32)
                }).collect::<HashMap<_, _>>()
            })
            -> map(|v: HashMap<u64, i32>| Bytes::from(serde_json::to_string(&v).unwrap()))
            -> dest_sink(query_responses);
    };

    let f1_handle = tokio::spawn(f1);
    hydroflow::util::cli::launch_flow(df).await;
    f1_handle.abort();
}
