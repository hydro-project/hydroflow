use std::io;
use futures::Stream;
use futures::Sink;
use hydroflow::bytes::{BytesMut, Bytes};
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::cli::{ConnectedDirect, ConnectedDemux, ConnectedTagged, ConnectedSink, ConnectedSource};
use hydroflow::hydroflow_syntax;
use hydroflow::util::collect_ready;
use hydroflow::util::collect_ready_async;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Payload {
    timestamp: usize,
    data: usize
} 


fn run_topolotree<S: Sink<(u32, Bytes)> + Unpin + 'static>(neighbors: Vec<u32>, input_recv: impl Stream<Item = Result<(u32, BytesMut), io::Error>> + Unpin + 'static, output_send: S ) -> Hydroflow
where <S as futures::Sink<(u32, hydroflow::bytes::Bytes)>>::Error: Debug{


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

        input 
            -> map(|(src, payload)| (src, (payload, context.current_tick())))
            -> inspect(|x| eprintln!("input: {:?}", x))
            -> all_neighbor_data;
        
        all_neighbor_data = reduce_keyed(|acc:&mut (Payload, usize), val:(Payload, usize)| {
                if val.0.timestamp > acc.0.timestamp {
                    *acc = val;
                }
            })
            -> persist();
        
        // Cross Join
        neighbors = source_iter(neighbors) -> persist();
        all_neighbor_data -> [0]aggregated_data;
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

    hydroflow::util::cli::launch_flow(run_topolotree(neighbors, input_recv, output_send)).await;
}


#[hydroflow::test]
async fn simple_test() {
    // let args: Vec<String> = std::env::args().skip(1).collect();
    let neighbors: Vec<u32> = vec![1,2]; //args.into_iter().map(|x| x.parse().unwrap()).collect();
    // let current_id = neighbors[0];

    let (input_send, input_recv) = hydroflow::util::unbounded_channel::<Result<(u32, BytesMut), io::Error>>();
    let (output_send, mut output_recv) = futures::channel::mpsc::unbounded::<(u32, Bytes)>();

    let mut flow: Hydroflow = run_topolotree(neighbors, input_recv, output_send);

    let simulate_input = |(id, payload): (u32, Payload)| {
        input_send.send(Ok((id, BytesMut::from(serde_json::to_string(&payload).unwrap().as_str()))))
    };

    let mut receive_all_output = || async move {
        let collected = collect_ready_async::<Vec<_>, _>(&mut output_recv).await;
        collected.iter().map(|(id, bytes)| (*id, serde_json::from_slice::<Payload>(&bytes[..]).unwrap())).collect::<Vec<_>>()
    };

    simulate_input((1, Payload {
        timestamp: 1,
        data: 2
    })).unwrap();

    flow.run_tick();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;


    assert_eq!(receive_all_output().await, &[(2, Payload {timestamp:1, data:2})]);
}
