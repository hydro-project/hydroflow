use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::{KvsMessageWithAddr, KvsResponse};
use crate::GraphType;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, graph: Option<GraphType>) {
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = union() -> dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound)
            -> map(Result::unwrap)
            -> map(|(msg, addr)| KvsMessageWithAddr::from_message(msg, addr))
            -> demux_enum::<KvsMessageWithAddr>();
        puts = inbound_chan[Put] -> tee();
        gets = inbound_chan[Get] -> tee();

        puts -> for_each(|(key, value, addr)| println!("Got a Put {:?}->{:?} from {:?}", key, value, addr));
        gets -> for_each(|(key, addr)| println!("Got a Get {:?} from {:?}", key, addr));

        // ack puts
        puts -> map(|(key, value, client_addr)| (KvsResponse { key, value }, client_addr)) -> [0]outbound_chan;

        // join PUTs and GETs by key
        puts -> map(|(key, value, _addr)| (key, value)) -> [0]lookup;
        gets -> [1]lookup;
        lookup = join::<'static>() -> tee();
        lookup[0] -> for_each(|t| println!("Found a match: {:?}", t));

        // send lookup responses back to the client address from the GET
        lookup[1] -> map(|(key, (value, client_addr))| (KvsResponse { key, value }, client_addr)) -> [1]outbound_chan;
    };

    if let Some(graph) = graph {
        let serde_graph = df
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
