use hydroflow::bytes::Bytes;
use hydroflow::hydroflow_syntax;
use hydroflow::tokio;
use hydroflow::util::cli::ConnectedDemux;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes};

enum GossipOrIncrement {
    Gossip(Vec<u32>),
    Increment(u32),
}

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let my_id: Vec<usize> = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let my_id = my_id[0];
    let num_peers: Vec<usize> = serde_json::from_str(&std::env::args().nth(2).unwrap()).unwrap();
    let num_peers = num_peers[0];

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

    let from_peer = ports
        .remove("from_peer")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let to_peer = ports
        .remove("to_peer")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let mut df = hydroflow_syntax! {
        next_state = merge()
            -> fold::<'static>(Vec::new(), |mut cur_state, goi| {
                match goi {
                    GossipOrIncrement::Gossip(gossip) => {
                        for i in 0..cur_state.len() {
                            if gossip[i] > cur_state[i] {
                                cur_state[i] = gossip[i];
                            }
                        }
                    }
                    GossipOrIncrement::Increment(increment) => {
                        cur_state[my_id] += increment;
                    }
                }
                cur_state
            }) -> tee();

        source_stream(from_peer)
            -> map(|x| deserialize_from_bytes::<Vec<u32>>(x.unwrap().to_vec()).unwrap())
            -> map(|x| GossipOrIncrement::Gossip(x))
            -> next_state;

        next_state -> next_tick() -> map(|g| GossipOrIncrement::Gossip(g)) -> next_state;

        source_stream(increment_requests) //TODO implement
            -> map(|x| String::from_utf8(x.unwrap().to_vec()).unwrap())
            -> map(|x| serde_json::from_str::<u32>(&x).unwrap())
            -> map(|x| GossipOrIncrement::Increment(x))
            -> next_state;

        all_peers = repeat_iter(0..num_peers)
            -> filter(|x| *x != my_id);
        
        all_peers -> [0] broadcaster;
        next_state -> [1] broadcaster;
        broadcaster = cross_join()
            -> map(|(peer, state)| {
                (peer as u32, serialize_to_bytes(state))
            })
            -> dest_sink(to_peer);

        next_state
            -> map(|a: Vec<u32>| a.iter().sum())
            -> unique::<'static>()
            -> map(|v: u32| Bytes::from(serde_json::to_string(&v).unwrap()))
            -> dest_sink(query_responses);
    };

    df.run_async().await.unwrap();
}
