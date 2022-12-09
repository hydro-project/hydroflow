use crate::GraphType;
use hydroflow::pusherator::Pusherator;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::Message;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, graph: Option<GraphType>) {
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = merge() -> sink_async_serde(outbound);
        inbound_chan = recv_stream_serde(inbound)
            ->  demux(|(m, a), tl!(members, msgs, errs)|
                    match m {
                        Message::ConnectRequest => members.give(a),
                        Message::ChatMsg {..} => msgs.give(m),
                        _ => errs.give(m),
                    }
                );
        members = inbound_chan[members] -> tee();
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // Logic
        members[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]outbound_chan;
        broadcast = cross_join() -> [1]outbound_chan;
        inbound_chan[msgs] -> [0]broadcast;
        members[1] -> [1]broadcast;
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
