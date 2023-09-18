#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Debug};
use std::io;
use std::rc::Rc;

use futures::{SinkExt, Stream};
use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged,
};

mod protocol;
use hydroflow::util::{serialize_to_bytes, deserialize_from_bytes};
use protocol::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeID(pub u32);

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

fn run_topolotree(
    neighbors: Vec<u32>,
    input_recv: impl Stream<Item = Result<(u32, BytesMut), io::Error>> + Unpin + 'static,
    increment_requests: impl Stream<Item = Result<BytesMut, io::Error>> + Unpin + 'static,
    output_send: tokio::sync::mpsc::UnboundedSender<(u32, Bytes)>,
    query_send: tokio::sync::mpsc::UnboundedSender<Bytes>,
) -> Hydroflow {
    fn merge(x: &mut i64, y: i64) {
        *x += y;
    }

    // Timestamp stuff is a bit complicated, there is a proper data-flowy way to do it
    // but it would require at least one more join and one more cross join just specifically for the local timestamps
    // Until we need it to be proper then we can take a shortcut and use rc refcell
    let self_timestamp = Rc::new(RefCell::new(HashMap::<u64, isize>::new()));

    let self_timestamp1 = Rc::clone(&self_timestamp);
    let self_timestamp2 = Rc::clone(&self_timestamp);
    let self_timestamp3 = Rc::clone(&self_timestamp);

    // we use current tick to keep track of which *keys* have been modified

    hydroflow_syntax! {
        from_neighbors = source_stream(input_recv)
            -> map(Result::unwrap)
            -> map(|(src, x)| (NodeID(src), deserialize_from_bytes::<Payload<i64>>(&x).unwrap()));

        from_neighbors
            -> map(|(src, payload)| ((payload.key, src), (payload.key, payload.contents)))
            -> fold_keyed::<'static>(|| (Timestamped { timestamp: -1, data: Default::default() }, 0), |acc: &mut (Timestamped<i64>, usize), (key, val): (u64, Timestamped<i64>)| {
                if val.timestamp > acc.0.timestamp {
                    acc.0 = val;
                    *self_timestamp1.borrow_mut().entry(key).or_insert(0) += 1;
                    acc.1 = context.current_tick();
                }
            })
            -> map(|((key, src), (payload, change_tick))| ((key, Some(src)), (payload.data, change_tick)))
            -> from_neighbors_or_local;

        local_value = source_stream(increment_requests)
            -> map(|x| deserialize_from_bytes::<OperationPayload>(&x.unwrap()).unwrap())
            -> inspect(|change| {
                *self_timestamp2.borrow_mut().entry(change.key).or_insert(0) += 1;
            })
            -> map(|change_payload: OperationPayload| (change_payload.key, (change_payload.change, context.current_tick())))
            -> reduce_keyed::<'static>(|agg: &mut (i64, usize), change: (i64, usize)| {
                agg.0 += change.0;
                agg.1 = std::cmp::max(agg.1, change.1);
            });

        local_value -> map(|(key, data)| ((key, None), data)) -> from_neighbors_or_local;

        from_neighbors_or_local = union() -> tee();
        from_neighbors_or_local -> [0]all_neighbor_data;

        neighbors = source_iter(neighbors)
            -> map(NodeID)
            -> persist();

        neighbors -> [1]all_neighbor_data;

        query_result = from_neighbors_or_local
            -> map(|((key, _), payload): ((u64, _), (i64, usize))| {
                (key, payload)
            })
            -> reduce_keyed(|acc: &mut (i64, usize), (data, change_tick): (i64, usize)| {
                merge(&mut acc.0, data);
                acc.1 = std::cmp::max(acc.1, change_tick);
            })
            -> filter(|(_, (_, change_tick))| *change_tick == context.current_tick())
            -> for_each(|(key, (data, _))| {
                let serialized = serialize_to_bytes(QueryResponse {
                    key,
                    value: data
                });
                query_send.send(serialized).unwrap();
            });

        all_neighbor_data = cross_join_multiset()
            -> filter(|(((_, aggregate_from_this_guy), _), target_neighbor): &(((u64, Option<NodeID>), (i64, usize)), NodeID)| {
                aggregate_from_this_guy.iter().all(|source| source != target_neighbor)
            })
            -> map(|(((key, _), payload), target_neighbor)| {
                ((key, target_neighbor), payload)
            })
            -> reduce_keyed(|acc: &mut (i64, usize), (data, change_tick): (i64, usize)| {
                merge(&mut acc.0, data);
                acc.1 = std::cmp::max(acc.1, change_tick);
            })
            -> filter(|(_, (_, change_tick))| *change_tick == context.current_tick())
            -> map(|((key, target_neighbor), (data, _))| (target_neighbor, Payload {
                key,
                contents: Timestamped {
                    timestamp: self_timestamp3.borrow().get(&key).copied().unwrap_or(0),
                    data,
                }
            }))
            -> for_each(|(target_neighbor, output): (NodeID, Payload<i64>)| {
                let serialized = serialize_to_bytes(output);
                output_send.send((target_neighbor.0, serialized)).unwrap();
            });
    }
}

#[hydroflow::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = args.into_iter().map(|x| x.parse().unwrap()).collect();

    let mut ports = hydroflow::util::cli::init().await;

    let input_recv = ports
        .port("from_peer")
        // connect to the port with a single recipient
        .connect::<ConnectedTagged<ConnectedDirect>>()
        .await
        .into_source();

    let mut output_send = ports
        .port("to_peer")
        .connect::<ConnectedDemux<ConnectedDirect>>()
        .await
        .into_sink();

    let operations_send = ports
        .port("increment_requests")
        // connect to the port with a single recipient
        .connect::<ConnectedDirect>()
        .await
        .into_source();

    let mut query_responses = ports
        .port("query_responses")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    let (chan_tx, mut chan_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::task::spawn_local(async move {
        while let Some(msg) = chan_rx.recv().await {
            output_send.send(msg).await.unwrap();
        }
    });

    let (query_tx, mut query_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::task::spawn_local(async move {
        while let Some(msg) = query_rx.recv().await {
            query_responses.send(msg).await.unwrap();
        }
    });

    let flow = run_topolotree(
        neighbors,
        input_recv,
        operations_send,
        chan_tx,
        query_tx
    );

    let f1 = async move {
        #[cfg(target_os = "linux")]
        loop {
            let x = procinfo::pid::stat_self().unwrap();
            let bytes = x.rss * 1024 * 4;
            println!("memory,{}", bytes);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    };

    // initial memory
    #[cfg(target_os = "linux")]
    {
        let x = procinfo::pid::stat_self().unwrap();
        let bytes = x.rss * 1024 * 4;
        println!("memory,{}", bytes);
    }

    let f1_handle = tokio::spawn(f1);
    hydroflow::util::cli::launch_flow(flow).await;
    f1_handle.abort();
}
