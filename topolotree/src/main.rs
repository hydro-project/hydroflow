#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::fmt::Display;
use std::io;
use std::rc::Rc;

use futures::{SinkExt, Stream};
use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged,
};
use topolotree_datatypes::{OperationPayload, Payload};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeID(pub u32);

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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
    let self_timestamp = Rc::new(RefCell::new(0));

    let self_timestamp1 = Rc::clone(&self_timestamp);
    let self_timestamp2 = Rc::clone(&self_timestamp);
    let self_timestamp3 = Rc::clone(&self_timestamp);

    hydroflow_syntax! {
        from_neighbors = source_stream(input_recv)
            -> map(Result::unwrap)
            -> map(|(src, payload)| (NodeID(src), serde_json::from_slice(&payload[..]).unwrap()))
            -> inspect(|(src, payload): &(NodeID, Payload<i64>)| println!("received from: {src}: payload: {payload:?}"));

        from_neighbors
            -> fold_keyed::<'static>(|| Payload { timestamp: -1, data: Default::default() }, |acc: &mut Payload<i64>, val: Payload<i64>| {
                if val.timestamp > acc.timestamp {
                    *acc = val;
                    *self_timestamp1.borrow_mut() += 1;
                }
            })
            -> inspect(|(src, data)| println!("data from stream: {src}: data: {data:?}"))
            -> map(|(src, payload)| (Some(src), payload.data))
            -> from_neighbors_or_local;

        local_value = source_stream(increment_requests)
            -> map(Result::unwrap)
            -> map(|change_payload: BytesMut| (serde_json::from_slice(&change_payload[..]).unwrap()))
            -> inspect(|change_payload: &OperationPayload| println!("change: {change_payload:?}"))
            -> inspect(|_| {
                *self_timestamp2.borrow_mut() += 1;
            })
            -> map(|change_payload: OperationPayload| change_payload.change)
            -> reduce::<'static>(|agg: &mut i64, change: i64| *agg += change);

        local_value -> map(|data| (None, data)) -> from_neighbors_or_local;

        from_neighbors_or_local = union();
        from_neighbors_or_local -> [0]all_neighbor_data;

        neighbors = source_iter(neighbors)
            -> map(NodeID)
            -> persist();

        neighbors -> [1]all_neighbor_data;

        all_neighbor_data = cross_join_multiset()
            -> filter(|((aggregate_from_this_guy, _), target_neighbor)| {
                aggregate_from_this_guy.iter().all(|source| source != target_neighbor)
            })
            -> map(|((_, payload), target_neighbor)| {
                (target_neighbor, payload)
            })
            -> fold_keyed(|| 0, |acc: &mut i64, data: i64| {
                merge(acc, data);
            })
            -> inspect(|(target_neighbor, data)| println!("data from neighbors: {target_neighbor:?}: data: {data:?}"))
            -> map(|(target_neighbor, data)| {
                (target_neighbor, Payload {
                    timestamp: *self_timestamp3.borrow(),
                    data
                })
            })
            -> for_each(|(target_neighbor, output): (NodeID, Payload<i64>)| {
                let serialized = BytesMut::from(serde_json::to_string(&output).unwrap().as_str()).freeze();
                output_send.send((target_neighbor.0, serialized)).unwrap();
            });
    }
}

#[hydroflow::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = args.into_iter().map(|x| x.parse().unwrap()).collect();
    // let current_id = neighbors[0];

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

    hydroflow::util::cli::launch_flow(run_topolotree(
        neighbors,
        input_recv,
        operations_send,
        chan_tx,
        query_tx
    ))
    .await;
}
