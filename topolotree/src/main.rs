#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::fmt::Debug;
use std::io;
use std::rc::Rc;

use futures::{SinkExt, Stream};
use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash)]
pub struct Payload<T: Debug> {
    timestamp: isize,
    data: T,
}

impl<T: PartialEq + Debug> PartialEq for Payload<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.timestamp == other.timestamp {
            assert_eq!(self.data, other.data);
            true
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OperationPayload {
    change: i64,
}

fn run_topolotree(
    neighbors: Vec<u32>,
    input_recv: impl Stream<Item = Result<(u32, BytesMut), io::Error>> + Unpin + 'static,
    local_update_recv: impl Stream<Item = Result<BytesMut, io::Error>> + Unpin + 'static,
    output_send: tokio::sync::mpsc::UnboundedSender<(u32, Bytes)>,
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
            -> map(|(src, payload)| (src, serde_json::from_slice(&payload[..]).unwrap()))
            -> inspect(|(src, payload): &(u32, Payload<i64>)| println!("received from: {src}: payload: {payload:?}"))
            -> tee();

        from_neighbors
            -> persist()
            -> fold_keyed(|| Payload { timestamp: -1, data: Default::default() }, |acc: &mut Payload<i64>, val: Payload<i64>| {
                if val.timestamp > acc.timestamp {
                    *acc = val;
                    *self_timestamp1.borrow_mut() += 1;
                }
            })
            -> inspect(|(src, data)| println!("data from stream: {src}: data: {data:?}"))
            -> [0]all_neighbor_data;

        local_value = source_stream(local_update_recv)
            -> map(Result::unwrap)
            -> map(|change_payload: BytesMut| (serde_json::from_slice(&change_payload[..]).unwrap()))
            -> inspect(|change_payload: &OperationPayload| println!("change: {change_payload:?}"))
            -> inspect(|_| {
                *self_timestamp2.borrow_mut() += 1;
            })
            -> persist()
            -> fold(0, |agg: &mut i64, op: OperationPayload| *agg += op.change);

        neighbors = source_iter(neighbors)
            -> persist()
            -> tee();

        // [1, 2, 3] + SelfState
        // message comes in from 2
        // (2+3+SelfState) -> 1, (1+2+SelfState) -> 3

        from_neighbors // 2 comes out here
            -> map(|(src, _payload)| src)
            -> [0]all_other_neighbors_except_for_who_it_came_from; // 2 goes into this crossjoin

        neighbors
            -> [1]all_other_neighbors_except_for_who_it_came_from;

        // (2, 1), (2, 2), (2, 3)
        all_other_neighbors_except_for_who_it_came_from = cross_join_multiset()
            -> filter(|(src, neighbor)| {
                src != neighbor
            })
            -> [0]who_to_aggregate_from_by_target; // (2, 1), (2, 3)

        neighbors
            -> [1]who_to_aggregate_from_by_target;

        // ((2, 1), 1)), ((2, 1), 2)), ((2, 1), 3)),
        // ((2, 3), 1)), ((2, 3), 2)), ((2, 3), 3)),
        who_to_aggregate_from_by_target = cross_join_multiset()
            -> filter(|((_original_src, target_neighbor), aggregate_from_this_guy)| {
                target_neighbor != aggregate_from_this_guy
            })
            // ((2, 1), 2)), ((2, 1), 3)),
            // ((2, 3), 1)), ((2, 3), 2)),
            -> map(|((original_src, target_neighbor), aggregate_from_this_guy)| {
                (aggregate_from_this_guy, (original_src, target_neighbor))
            })
            // (2, (2, 1))), (3, (2, 1))),
            // (1, (2, 3))), (2, (2, 3))),
            -> [1]all_neighbor_data;

        all_neighbor_data = join()
            -> map(|(aggregate_from_this_guy, (payload, (original_src, target_neighbor)))| {
                ((target_neighbor, original_src), (aggregate_from_this_guy, payload))
            })
            -> fold_keyed(|| 0, |acc: &mut i64, (_aggregate_from_this_guy, payload): (u32, Payload<i64>)| {
                merge(acc, payload.data);
            })
            -> [0]add_local_value;

        local_value
            -> [1]add_local_value;

        add_local_value = cross_join_multiset()
            -> map(|(((target_neighbor, _original_src), data), local_value)| {
                (target_neighbor, Payload {
                    timestamp: *self_timestamp3.borrow(),
                    data: data + local_value
                })
            })
            -> for_each(|(target_neighbor, output)| {
                let serialized = BytesMut::from(serde_json::to_string(&output).unwrap().as_str()).freeze();
                output_send.send((target_neighbor, serialized)).unwrap();
            });

        // src
        //     -> map(|(from, _data)| from)
        //     -> enumerate()
        //     -> [0]cj1;

        // source_iter(NEIGHBORS)
        //     -> persist()
        //     -> [1]cj1;

        // cj1 = cross_join::<HalfMultisetJoinState>()
        //     -> filter(|((_req_id, from), to)| to != from)
        //     -> map(|((req_id, _from), to)| (to, req_id))
        //     -> [0]cj2;

        // source_iter(NEIGHBORS)
        //     -> persist()
        //     -> [1]cj2;

        // cj2 = cross_join::<HalfMultisetJoinState>()
        //     -> filter(|((to, _req_id), node_id)| node_id != to)
        //     -> map(|((to, req_id), node_id)| (node_id, (req_id, to)))
        //     -> [0]j;



        // all_neighbor_data -> neighbors_and_myself;
        // operations_input -> fold::<'static>(0, |agg: &mut i64, op: i64| *agg += op) -> map(|total| (my_id, total)) -> neighbors_and_myself;
        // neighbors_and_myself = union();

        // // Cross Join
        // neighbors = source_iter(neighbors) -> persist();
        // neighbors_and_myself -> [0]aggregated_data;
        // neighbors -> [1]aggregated_data;

        // // (dest, Payload) where Payload has timestamp and accumulated data as specified by merge function
        // aggregated_data = cross_join_multiset()
        //     -> filter(|((src, (payload, tick)), dst)| src != dst)
        //     -> map(|((src, (payload, tick)), dst)| (dst, payload));

        // aggregated_data
        //     -> map(|(dst, payload)| (dst, BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()).freeze()))
        //     -> for_each(|x| {
        //         output_send.send(x).unwrap();
        //     });

    }
}

#[hydroflow::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = args.into_iter().map(|x| x.parse().unwrap()).collect();
    // let current_id = neighbors[0];

    let mut ports = hydroflow::util::cli::init().await;

    let input_recv = ports
        .port("input")
        // connect to the port with a single recipient
        .connect::<ConnectedTagged<ConnectedDirect>>()
        .await
        .into_source();

    let mut output_send = ports
        .port("output")
        .connect::<ConnectedDemux<ConnectedDirect>>()
        .await
        .into_sink();

    let operations_send = ports
        .port("input")
        // connect to the port with a single recipient
        .connect::<ConnectedDirect>()
        .await
        .into_source();

    let _increment_requests = ports
        .port("increment_requests")
        .connect::<ConnectedDirect>()
        .await
        .into_source();

    let _query_responses = ports
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

    hydroflow::util::cli::launch_flow(run_topolotree(
        neighbors,
        input_recv,
        operations_send,
        chan_tx,
    ))
    .await;
}
