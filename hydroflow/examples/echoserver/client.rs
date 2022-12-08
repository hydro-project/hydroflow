use chrono::prelude::*;
use std::net::SocketAddr;

use crate::protocol::EchoMsg;
use crate::Opts;
use hydroflow::hydroflow_syntax;

pub(crate) async fn run_client(opts: Opts, server_addr: SocketAddr, my_addr: SocketAddr) {
    println!("Attempting to connect to server at {}", server_addr);

    println!("{:?} live!", opts.role);

    let mut flow = hydroflow_syntax! {
        // set up channels
        outbound_chan = sink_udp(0);
        inbound_chan = recv_udp(my_addr.port()) -> map(|(m, _a)| m);

        // take stdin and send to server as an Echo::Message
        lines = recv_stdin() -> map(|l| (EchoMsg{ payload: l.unwrap(), ts: Utc::now(), }, server_addr))
            -> outbound_chan;

        // receive and print messages
        inbound_chan[msgs] -> for_each(|m: EchoMsg| println!("{:?}", m));
    };

    flow.run_async().await.unwrap();
}
