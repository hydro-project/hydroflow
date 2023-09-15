use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io;
use std::rc::Rc;

use futures::{Sink, SinkExt, Stream};
use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged,
};
use hydroflow::util::multiset::HashMultiSet;
use hydroflow::util::{collect_ready_async, unbounded_channel};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash)]
struct Payload<T: Debug> {
    timestamp: isize,
    data: T,
}

impl<T: Debug> Payload<T> {
    pub fn merge_from(&mut self, other: Payload<T>) -> bool {
        if other.timestamp > self.timestamp {
            self.data = other.data;
            self.timestamp = other.timestamp;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, updater: impl Fn(&T) -> T) {
        self.data = updater(&self.data);
        self.timestamp += 1;
    }
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
struct OperationPayload {
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

    let increment_requests = ports
        .port("increment_requests")
        .connect::<ConnectedDirect>()
        .await
        .into_source();

    let query_responses = ports
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

#[hydroflow::test]
async fn simple_payload_test() {
    // let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = vec![1, 2, 3]; // args.into_iter().map(|x| x.parse().unwrap()).collect();
                                             // let current_id = neighbors[0];

    let (operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();
    let (input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let input1 = (
        1,
        Payload {
            timestamp: 1,
            data: 2,
        },
    );
    // let input2 = (1, Payload {timestamp:1, data:3});
    // let payload_vec = vec![input1, input2];
    // let payload_stream = stream::iter(payload_vec).map(|(i, payload)| Ok((i, BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()))));
    let simulate_input = |(id, payload): (u32, Payload<i64>)| {
        input_send.send(Ok((
            id,
            BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
        )))
    };
    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, operations_rx, output_send);
    let receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected
            .iter()
            .map(|(id, bytes)| {
                (
                    *id,
                    serde_json::from_slice::<Payload<i64>>(&bytes[..]).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    simulate_input(input1).unwrap();
    flow.run_tick();
    let output1: (u32, Payload<i64>) = (
        2,
        Payload {
            timestamp: 1,
            data: 2,
        },
    );
    let output2: (u32, Payload<i64>) = (
        3,
        Payload {
            timestamp: 1,
            data: 2,
        },
    );
    assert_eq!(receive_all_output().await, &[output1, output2]);
}

#[hydroflow::test]
async fn idempotence_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];
    let (operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();

    let (input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let input1 = (
        1,
        Payload {
            timestamp: 4,
            data: 2,
        },
    );
    let input2 = (
        1,
        Payload {
            timestamp: 4,
            data: 2,
        },
    );
    let simulate_input = |(id, payload): (u32, Payload<i64>)| {
        input_send.send(Ok((
            id,
            BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
        )))
    };
    simulate_input(input1).unwrap();
    simulate_input(input2).unwrap();
    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, operations_rx, output_send);
    let receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected
            .iter()
            .map(|(id, bytes)| {
                (
                    *id,
                    serde_json::from_slice::<Payload<i64>>(&bytes[..]).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    flow.run_tick();
    let output1: (u32, Payload<i64>) = (
        2,
        Payload {
            timestamp: 1,
            data: 2,
        },
    );
    let output2: (u32, Payload<i64>) = (
        3,
        Payload {
            timestamp: 1,
            data: 2,
        },
    );
    assert_eq!(receive_all_output().await, &[output1, output2]);
}

#[hydroflow::test]
async fn backwards_in_time_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];

    let (operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();
    let (input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let input1 = (
        1,
        Payload {
            timestamp: 5,
            data: 7,
        },
    );
    let input2 = (
        1,
        Payload {
            timestamp: 4,
            data: 2,
        },
    );
    let simulate_input = |(id, payload): (u32, Payload<i64>)| {
        input_send.send(Ok((
            id,
            BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
        )))
    };
    simulate_input(input1).unwrap();
    simulate_input(input2).unwrap();
    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, operations_rx, output_send);
    let receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected
            .iter()
            .map(|(id, bytes)| {
                (
                    *id,
                    serde_json::from_slice::<Payload<i64>>(&bytes[..]).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    flow.run_tick();
    let output1: (u32, Payload<i64>) = (
        2,
        Payload {
            timestamp: 1,
            data: 7,
        },
    );
    let output2: (u32, Payload<i64>) = (
        3,
        Payload {
            timestamp: 1,
            data: 7,
        },
    );
    assert_eq!(receive_all_output().await, &[output1, output2]);
}

#[hydroflow::test]
async fn multiple_input_sources_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];
    let (operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();

    let (input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let input1 = (
        1,
        Payload {
            timestamp: 5,
            data: 7,
        },
    );
    let input2 = (
        2,
        Payload {
            timestamp: 4,
            data: 2,
        },
    );
    let simulate_input = |(id, payload): (u32, Payload<i64>)| {
        input_send.send(Ok((
            id,
            BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
        )))
    };
    simulate_input(input1).unwrap();
    simulate_input(input2).unwrap();
    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, operations_rx, output_send);
    let receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected
            .iter()
            .map(|(id, bytes)| {
                (
                    *id,
                    serde_json::from_slice::<Payload<i64>>(&bytes[..]).unwrap(),
                )
            })
            .collect::<HashMultiSet<_>>()
    };
    flow.run_tick();
    let output1 = (
        1,
        Payload {
            timestamp: 2,
            data: 2,
        },
    );
    let output2 = (
        2,
        Payload {
            timestamp: 2,
            data: 7,
        },
    );
    let output3 = (
        3,
        Payload {
            timestamp: 2,
            data: 9,
        },
    );
    assert_eq!(
        receive_all_output().await,
        HashMultiSet::from_iter([output1, output2, output3.clone(), output3])
    );

    // {(1, Payload { timestamp: 70, data: 2 }), (2, Payload { timestamp: 70, data: 7 }), (3, Payload { timestamp: 70, data: 18 })}
    // {(2, Payload { timestamp: 2, data: 7 }), (1, Payload { timestamp: 2, data: 2 }), (3, Payload { timestamp: 2, data: 9 })}
}

#[hydroflow::test]
async fn simple_operation_test() {
    // let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = vec![1, 2, 3]; // args.into_iter().map(|x| x.parse().unwrap()).collect();
                                             // let current_id = neighbors[0];

    let (operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();
    let (input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let input1 = (
        1,
        Payload {
            timestamp: 1,
            data: 2,
        },
    );

    operations_tx
        .send(Ok(BytesMut::from(
            serde_json::to_string(&OperationPayload { change: 5 })
                .unwrap()
                .as_str(),
        )))
        .unwrap();

    operations_tx
        .send(Ok(BytesMut::from(
            serde_json::to_string(&OperationPayload { change: 7 })
                .unwrap()
                .as_str(),
        )))
        .unwrap();

    let simulate_input = |(id, payload): (u32, Payload<i64>)| {
        input_send.send(Ok((
            id,
            BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
        )))
    };
    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, operations_rx, output_send);
    let receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected
            .iter()
            .map(|(id, bytes)| {
                (
                    *id,
                    serde_json::from_slice::<Payload<i64>>(&bytes[..]).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    simulate_input(input1).unwrap();
    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(receive_all_output().await, &[
        (2, Payload { timestamp: 3, data: 14 }),
        (3, Payload { timestamp: 3, data: 14 }),
    ]);
}

// idempotence test (issue two requests with the same timestamp and see that they don't change anything.)
//     let input1 = (1, Payload {timestamp:4, data:2});
//     let input2 = (1, Payload {timestamp:4, data:2});
//     let output1: (u32, Payload) = (2, Payload {timestamp:1, data:2});
//     let output2: (u32, Payload) = (3, Payload {timestamp:1, data:2});
//
// backward in time test (issue two requests, the second one with an earlier timestamp than the first. )
//     let input1 = (1, Payload {timestamp:5, data:7});
//     let input2 = (1, Payload {timestamp:4, data:2});
//     let output1: (u32, Payload) = (2, Payload {timestamp:1, data:7});
//     let output2: (u32, Payload) = (3, Payload {timestamp:1, data:7});
//
// updates from multiple sources test
//     let input1 = (1, Payload {timestamp:5, data:7});
//     let input2 = (2, Payload {timestamp:4, data:2});
//     let output1: (u32, Payload) = (1, Payload {timestamp:2, data:2});
//     let output2: (u32, Payload) = (2, Payload {timestamp:2, data:7});
//     let output3: (u32, Payload) = (3, Payload {timestamp:2, data:9});
