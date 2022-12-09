use chrono::prelude::*;
use std::net::SocketAddr;

use crate::protocol::EchoMsg;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream) {
    println!("Server live!");

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Inbound channel sharing
        inbound_chan = recv_stream_serde(inbound) -> tee();

        // Logic
        inbound_chan[0] -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("Got {:?} from {:?}", m, a));
        inbound_chan[1] -> map(|(EchoMsg { payload, .. }, addr)| (EchoMsg { payload, ts: Utc::now() }, addr))
            -> sink_async_serde(outbound);
    };
    flow.run_async().await.unwrap();
}
