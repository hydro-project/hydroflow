use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::{KvsMessageWithAddr, KvsResponse};
use crate::GraphType;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, graph: Option<GraphType>) {
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // network channels
        network_send = union() -> dest_sink_serde(outbound);
        network_recv = source_stream_serde(inbound)
            -> _upcast(Some(Delta))
            -> map(Result::unwrap)
            -> inspect(|(msg, addr)| println!("Message received {:?} from {:?}", msg, addr))
            -> map(|(msg, addr)| KvsMessageWithAddr::from_message(msg, addr))
            -> demux_enum::<KvsMessageWithAddr>();
        puts = network_recv[Put] -> tee();
        gets = network_recv[Get];

        // ack puts
        puts -> map(|(key, value, client_addr)| (KvsResponse { key, value }, client_addr)) -> [0]network_send;

        // join PUTs and GETs by key
        puts -> map(|(key, value, _addr)| (key, value)) -> persist() -> [0]lookup;
        gets -> [1]lookup;
        lookup = join::<'tick>();

        // network_send lookup responses back to the client address from the GET
        lookup[1]
            -> inspect(|tup| println!("Found a match: {:?}", tup))
            -> map(|(key, (value, client_addr))| (KvsResponse { key, value }, client_addr))
            -> [1]network_send;
    };

    if let Some(graph) = graph {
        let serde_graph = df
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                serde_graph.open_mermaid().unwrap();
            }
            GraphType::Dot => {
                serde_graph.open_dot().unwrap();
            }
            GraphType::Json => {
                unimplemented!();
            }
        }
    }

    df.run_async().await.unwrap();
}
