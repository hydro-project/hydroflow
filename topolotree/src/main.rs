use std::fmt::Debug;
use std::io;

use futures::{Sink, Stream, stream, StreamExt};
use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged,
};
use hydroflow::util::{collect_ready, collect_ready_async};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Payload {
    timestamp: usize,
    data: usize,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct OperationPayload {
    change: usize
}

fn run_topolotree<S: Sink<(u32, Bytes)> + Unpin + 'static>(
    neighbors: Vec<u32>,
    input_recv: impl Stream<Item = Result<(u32, BytesMut), io::Error>> + Unpin + 'static,
    local_update_recv: impl Stream<Item = Result<BytesMut, io::Error>> + Unpin + 'static,
    output_send: S,
) -> Hydroflow
where
    <S as futures::Sink<(u32, hydroflow::bytes::Bytes)>>::Error: Debug,
{
    fn merge(x: &mut usize, y: usize) {
        *x += y;
    }

    hydroflow_syntax! {
        input = union() -> tee();

        source_stream(input_recv)
            -> map(Result::unwrap)
            -> map(|(src, payload): (u32, BytesMut)| (src, serde_json::from_slice::<Payload>(&payload[..]).unwrap()))
            -> inspect(|(src, payload)| println!("received from: {src}: payload: {payload:?}"))
            -> input;

        source_stream(local_update_recv)
            -> map(Result::unwrap)
            -> map(|changePayload: BytesMut| (serde_json::from_slice::<OperationPayload>(&changePayload[..]).unwrap()))
            -> inspect(|changePayload| println!("change: {changePayload:?}"))
            // -> tee();
            -> operations_input;

        input
            -> map(|(src, payload)| (src, (payload, context.current_tick())))
            -> inspect(|x| eprintln!("input: {:?}", x))
            -> all_neighbor_data;

        all_neighbor_data = reduce_keyed::<'static>(|acc:&mut (Payload, usize), val:(Payload, usize)| {
                if val.0.timestamp > acc.0.timestamp {
                    *acc = val;
                }
            })
            -> inspect(|(src, payload)| println!("data from stream: {src}: payload: {payload:?}"));
            // -> persist();

        all_neighbor_data -> neighbors_and_myself;
        operations_input -> fold::<'static>(0, |agg: &mut i64, op: i64| *agg += op) -> map(|total| (my_id, total)) -> neighbors_and_myself;
        neighbors_and_myself = union();

        // Cross Join
        neighbors = source_iter(neighbors) -> persist();
        neighbors_and_myself -> [0]aggregated_data;
        neighbors -> [1]aggregated_data;

        // (dest, Payload) where Payload has timestamp and accumulated data as specified by merge function
        aggregated_data = cross_join_multiset()
            -> filter(|((src, (payload, tick)), dst)| src != dst)
            -> map(|((src, (payload, tick)), dst)| (dst, payload));


        aggregated_data
            -> map(|(dst, payload)| (dst, BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()).freeze()))
            -> dest_sink(output_send);

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

    let output_send = ports
        .port("output")
        .connect::<ConnectedDemux<ConnectedDirect>>()
        .await
        .into_sink();

    let operations_send = ports
        .port("input")
        // connect to the port with a single recipient
        .connect::<ConnectedTagged<ConnectedDirect>>()
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

    hydroflow::util::cli::launch_flow(run_topolotree(neighbors, input_recv, operations_send, output_send)).await;
}

#[hydroflow::test]
async fn simple_payload__test() {
    // let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = vec![1, 2, 3]; // args.into_iter().map(|x| x.parse().unwrap()).collect();
                                          // let current_id = neighbors[0];

    let (input_send, input_recv) =
        hydroflow::util::unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = futures::channel::mpsc::unbounded::<(u32, Bytes)>();


    let input1 = (1, Payload {timestamp:1, data:2});
    // let input2 = (1, Payload {timestamp:1, data:3});
    // let payload_vec = vec![input1, input2];
    // let payload_stream = stream::iter(payload_vec).map(|(i, payload)| Ok((i, BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()))));

    // Send (id, Payload) over network to neighbors
    let simulate_input = |(id, payload): (u32, Payload)| {
        input_send.send(Ok((
            id,
            BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()),
        )))
    };

    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, operations_send, output_send);

    let receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected
            .iter()
            .map(|(id, bytes)| (*id, serde_json::from_slice::<Payload>(&bytes[..]).unwrap()))
            .collect::<Vec<_>>()
    };

    simulate_input(input1).unwrap();

    flow.run_tick();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let output1: (u32, Payload) = (2, Payload {timestamp:1, data:2});
    let output2: (u32, Payload) = (3, Payload {timestamp:1, data:2});

    assert_eq!(
        receive_all_output().await,
        &[output1, output2]
    );
}
