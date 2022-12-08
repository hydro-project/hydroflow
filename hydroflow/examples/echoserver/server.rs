use crate::Opts;
use chrono::prelude::*;
use std::net::SocketAddr;

use crate::protocol::EchoMsg;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;

pub(crate) async fn run_server(opts: Opts, server_addr: SocketAddr) {
    println!("Listening on {}", server_addr);

    println!("{:?} live!", opts.role);

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Inbound channel sharing
        inbound_chan = recv_udp(server_addr.port()) -> tee();

        // Logic
        inbound_chan[0] -> for_each(|(m, a)| println!("Got {:?} from {:?}", m, a));
        inbound_chan[1] -> map(|(EchoMsg { payload, .. }, client)| (EchoMsg { payload, ts: Utc::now() }, client))
            -> sink_udp(0);
    };
    flow.run_async().await.unwrap();
}
