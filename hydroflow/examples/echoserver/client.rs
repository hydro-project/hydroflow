use chrono::prelude::*;

use std::net::SocketAddr;

use crate::protocol::EchoMsg;
use crate::Opts;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{bind_udp_socket, ipv4_resolve};

pub(crate) async fn run_client(opts: Opts) {
    println!("Attempting to connect to server at {}", opts.server_addr);
    println!("{:?} live!", opts.role);

    let (outbound, inbound) = bind_udp_socket(opts.addr.unwrap()).await;
    let server_addr = ipv4_resolve(opts.server_addr);

    let mut flow = hydroflow_syntax! {
        // // set up channels
        outbound_chan =  sink_async_serde(outbound);
        inbound_chan = recv_stream_serde(inbound);

        // take stdin and send to server as an Echo::Message
        lines = recv_stdin() -> map(|l| (EchoMsg{ payload: l.unwrap(), ts: Utc::now(), }, server_addr) )
            -> outbound_chan;

        // // receive and print messages
        inbound_chan -> for_each(|(m, _a): (EchoMsg, SocketAddr) | println!("{:?}", m));
    };

    flow.run_async().await.unwrap();
}
