use crate::Opts;
use chrono::prelude::*;
use std::net::SocketAddr;

use crate::protocol::EchoMsg;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::bind_udp_socket;

pub(crate) async fn run_server(opts: Opts) {
    println!("Listening on {}", opts.server_addr);
    let (outbound, inbound) = bind_udp_socket(opts.server_addr).await;

    println!("{:?} live!", opts.role);

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Inbound channel sharing
        socket = recv_stream_serde(inbound);
        inbound_chan = socket -> tee();

        // Logic
        inbound_chan[0] -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("Got {:?} from {:?}", m, a));
        inbound_chan[1] -> map(|(EchoMsg { payload, .. }, addr)| (EchoMsg { payload, ts: Utc::now() }, addr))
            -> sink_async_serde(outbound);
    };
    flow.run_async().await.unwrap();
}
