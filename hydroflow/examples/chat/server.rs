use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::{Message, MessageWithAddr};
use crate::Opts;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, opts: Opts) {
    println!("Server live!");

    let mut hf: Hydroflow = hydroflow_syntax! {
        // Define shared inbound and outbound channels
        outbound_chan = union() -> dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound)
            -> map(Result::unwrap)
            -> map(|(msg, addr)| MessageWithAddr::from_message(msg, addr))
            -> demux_enum::<MessageWithAddr>();
        clients = inbound_chan[ConnectRequest] -> map(|(addr,)| addr) -> tee();
        inbound_chan[ConnectResponse] -> for_each(|(addr,)| println!("Received unexpected `ConnectResponse` as server from addr {}.", addr));

        // Pipeline 1: Acknowledge client connections
        clients[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]outbound_chan;

        // Pipeline 2: Broadcast messages to all clients
        inbound_chan[ChatMsg] -> map(|(_addr, nickname, message, ts)| Message::ChatMsg { nickname, message, ts }) -> [0]broadcast;
        clients[1] -> [1]broadcast;
        broadcast = cross_join::<'tick, 'static>() -> [1]outbound_chan;
    };

    if let Some(graph) = opts.graph {
        let serde_graph = hf
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        serde_graph.open_graph(graph, opts.write_config).unwrap();
    }

    hf.run_async().await.unwrap();
}
