use crate::GraphType;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::Message;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, graph: Option<GraphType>) {
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = merge() -> dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound)
            ->  demux(|(msg, addr), tl!(clients, msgs, errs)|
                    match msg {
                        Message::ConnectRequest => clients.give(addr),
                        Message::ChatMsg {..} => msgs.give(msg),
                        _ => errs.give(msg),
                    }
                );
        clients = inbound_chan[clients] -> tee();
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // Pipeline 1: Acknowledge client connections
        clients[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]outbound_chan;

        // Pipeline 2: Broadcast messages to all clients
        broadcast = cross_join() -> [1]outbound_chan;
        inbound_chan[msgs] -> [0]broadcast;
        clients[1] -> [1]broadcast;
    };

    if let Some(graph) = graph {
        let serde_graph = df
            .serde_graph()
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
