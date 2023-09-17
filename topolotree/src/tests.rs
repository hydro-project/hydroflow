use std::fmt::Debug;
use std::io;

use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow::util::multiset::HashMultiSet;
use hydroflow::util::{collect_ready_async, unbounded_channel};
use serde::Serialize;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;

use crate::{run_topolotree, OperationPayload, Payload};

pub fn simulate_input<T: Debug + Serialize>(
    input_send: &mut UnboundedSender<Result<(u32, BytesMut), std::io::Error>>,
    (id, payload): (u32, Payload<T>),
) -> Result<(), SendError<Result<(u32, BytesMut), std::io::Error>>> {
    input_send.send(Ok((
        id,
        BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
    )))
}

pub fn simulate_operation(
    input_send: &mut UnboundedSender<Result<BytesMut, std::io::Error>>,
    payload: OperationPayload,
) -> Result<(), SendError<Result<BytesMut, std::io::Error>>> {
    input_send.send(Ok(BytesMut::from(
        serde_json::to_string(&payload).unwrap().as_str(),
    )))
}

pub async fn read_all(
    mut output_recv: &mut UnboundedReceiverStream<(u32, Bytes)>,
) -> HashMultiSet<(u32, Payload<i64>)> {
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
}

#[hydroflow::test]
async fn simple_payload_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];

    let (_operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();
    let (mut input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let (query_send, mut query_recv) = unbounded_channel::<Bytes>();

    #[rustfmt::skip]
    simulate_input(&mut input_send, (1, Payload { timestamp: 1, data: 2 })).unwrap();

    let mut flow = run_topolotree(neighbors, input_recv, operations_rx, output_send, query_send);

    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(read_all(&mut output_recv).await, HashMultiSet::from_iter([
        (2, Payload { timestamp: 1, data: 2 }),
        (3, Payload { timestamp: 1, data: 2 }),
    ]));
}

#[hydroflow::test]
async fn idempotence_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];
    let (_operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();

    let (mut input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let (query_send, mut query_recv) = unbounded_channel::<Bytes>();

    #[rustfmt::skip]
    {
        simulate_input(&mut input_send, (1, Payload { timestamp: 4, data: 2 })).unwrap();
        simulate_input(&mut input_send, (1, Payload { timestamp: 4, data: 2 })).unwrap();
    };

    let mut flow = run_topolotree(neighbors, input_recv, operations_rx, output_send, query_send);

    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(read_all(&mut output_recv).await, HashMultiSet::from_iter([
        (2, Payload { timestamp: 1, data: 2 }),
        (3, Payload { timestamp: 1, data: 2 }),
    ]));
}

#[hydroflow::test]
async fn backwards_in_time_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];

    let (_operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();
    let (mut input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let (query_send, mut query_recv) = unbounded_channel::<Bytes>();

    #[rustfmt::skip]
    {
        simulate_input(&mut input_send, (1, Payload { timestamp: 5, data: 7 })).unwrap();
        simulate_input(&mut input_send, (1, Payload { timestamp: 4, data: 2 })).unwrap();
    };

    let mut flow = run_topolotree(neighbors, input_recv, operations_rx, output_send, query_send);

    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(read_all(&mut output_recv).await, HashMultiSet::from_iter([
        (2, Payload { timestamp: 1, data: 7 }),
        (3, Payload { timestamp: 1, data: 7 }),
    ]));
}

#[hydroflow::test]
async fn multiple_input_sources_test() {
    let neighbors: Vec<u32> = vec![1, 2, 3];
    let (_operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();

    let (mut input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let (query_send, mut query_recv) = unbounded_channel::<Bytes>();

    #[rustfmt::skip]
    {
        simulate_input(&mut input_send, (1, Payload { timestamp: 5, data: 7 })).unwrap();
        simulate_input(&mut input_send, (2, Payload { timestamp: 4, data: 2 })).unwrap();
    };

    let mut flow = run_topolotree(neighbors, input_recv, operations_rx, output_send, query_send);

    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(read_all(&mut output_recv).await, HashMultiSet::from_iter([
        (1, Payload { timestamp: 2, data: 2 }),
        (2, Payload { timestamp: 2, data: 7 }),
        (3, Payload { timestamp: 2, data: 9 }),
    ]));
}

#[hydroflow::test]
async fn operations_across_ticks() {
    let neighbors: Vec<u32> = vec![1, 2, 3];

    let (mut operations_tx, operations_rx) = unbounded_channel::<Result<BytesMut, io::Error>>();
    let (mut input_send, input_recv) = unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = unbounded_channel::<(u32, Bytes)>();
    let (query_send, mut query_recv) = unbounded_channel::<Bytes>();

    let mut flow = run_topolotree(neighbors, input_recv, operations_rx, output_send, query_send);

    #[rustfmt::skip]
    {
        simulate_input(&mut input_send, (1, Payload { timestamp: 1, data: 2 })).unwrap();
        simulate_operation(&mut operations_tx, OperationPayload { change: 5 }).unwrap();
        simulate_operation(&mut operations_tx, OperationPayload { change: 7 }).unwrap();
    };

    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(read_all(&mut output_recv).await, HashMultiSet::from_iter([
        (1, Payload { timestamp: 3, data: 12 }),
        (2, Payload { timestamp: 3, data: 14 }),
        (3, Payload { timestamp: 3, data: 14 }),
    ]));

    #[rustfmt::skip]
    {
        simulate_operation(&mut operations_tx, OperationPayload { change: 1 }).unwrap();
    };

    flow.run_tick();

    #[rustfmt::skip]
    assert_eq!(read_all(&mut output_recv).await, HashMultiSet::from_iter([
        (1, Payload { timestamp: 4, data: 13 }),
        (2, Payload { timestamp: 4, data: 15 }),
        (3, Payload { timestamp: 4, data: 15 }),
    ]));
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
