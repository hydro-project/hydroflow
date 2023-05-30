use crate::helpers::print_graph;
use crate::protocol::Message;
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, opts: crate::Opts) {
    println!("Server live!");

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Define shared inbound and outbound channels
        inbound_chan = source_stream_serde(inbound) -> map(|udp_msg| udp_msg.unwrap()) -> tee();
        outbound_chan = union() -> dest_sink_serde(outbound);

        // Print all messages for debugging purposes
        inbound_chan[1]
            -> for_each(|(m, a): (Message, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), m, a));

        // Demux and destructure the inbound messages into separate streams
        inbound_demuxed = inbound_chan[0]
            ->  demux(|(msg, addr), var_args!(echo, heartbeat, errs)|
                    match msg {
                        Message::Echo {payload, ..} => echo.give((payload, addr)),
                        Message::Heartbeat => heartbeat.give(addr),
                        _ => errs.give((msg, addr)),
                    }
                );

        // Echo back the Echo messages with updated timestamp
        inbound_demuxed[echo]
            -> map(|(payload, addr)| (Message::Echo { payload, ts: Utc::now() }, addr) ) -> [0]outbound_chan;

        // Respond to Heartbeat messages
        inbound_demuxed[heartbeat] -> map(|addr| (Message::HeartbeatAck, addr)) -> [1]outbound_chan;

        // Print unexpected messages
        inbound_demuxed[errs]
            -> for_each(|(msg, addr)| println!("Received unexpected message type: {:?} from {:?}", msg, addr));

    };

    if let Some(graph) = opts.graph {
        print_graph(&flow, graph);
    }

    // run the server
    flow.run_async().await;
}
