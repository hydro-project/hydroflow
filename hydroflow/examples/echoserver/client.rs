use crate::protocol::EchoMsg;
use crate::Opts;
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

pub(crate) async fn run_client(outbound: UdpSink, inbound: UdpStream, opts: Opts) {
    // server_addr is required for client
    let server_addr = opts.server_addr.expect("Client requires a server address");
    println!("Client live!");

    let mut flow = hydroflow_syntax! {
        // Define shared inbound and outbound channels
        inbound_chan = source_stream_serde(inbound)
            // -> tee() // commented out since we only use this once in the client template
        ;
        outbound_chan = // merge() ->  // commented out since we only use this once in the client template
            dest_sink_serde(outbound);

        // Print all messages for debugging purposes
        inbound_chan
            -> filter_map(bincode::Result::ok)
            -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), m, a));

        // take stdin and send to server as an Message::Echo
        source_stdin() -> map(|l| (EchoMsg{ payload: l.unwrap(), ts: Utc::now(), }, server_addr) )
            -> outbound_chan;
    };

    flow.run_async().await.unwrap();
}
