use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

use crate::protocol::Msg;

fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 2) + fibonacci(n - 1)
    }
}

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, _opts: crate::Opts) {
    println!("Server live!");

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Define a shared inbound channel
        inbound_chan = source_stream_serde(inbound) -> map(Result::unwrap);

        // Echo back the Echo messages with updated timestamp
        inbound_chan
            // Print all messages for debugging purposes
            -> inspect(|(msg, addr): &(Msg, SocketAddr)| println!("{}: Got {}=>{} from {:?}", Utc::now(), msg.idx, msg.val, addr))
            -> map(|(msg, addr): (Msg, SocketAddr)| (Msg { val: fibonacci(msg.val), ..msg }, addr) ) -> dest_sink_serde(outbound);
    };

    // run the server
    flow.run_async().await;
}
