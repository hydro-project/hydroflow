use std::net::SocketAddr;

use chrono::prelude::*;
use dfir_rs::dfir_syntax;
use dfir_rs::scheduled::graph::Dfir;
use dfir_rs::util::{UdpLinesSink, UdpLinesStream};

use crate::helpers::{deserialize_json, serialize_json};
use crate::protocol::EchoMsg;

pub(crate) async fn run_server(outbound: UdpLinesSink, inbound: UdpLinesStream) {
    println!("Server live!");

    let mut flow: Dfir = dfir_syntax! {
        // Inbound channel sharing
        inbound_chan = source_stream(inbound) -> map(deserialize_json) -> tee();

        // Logic
        inbound_chan[0] -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("Got {:?} from {:?}", m, a));
        inbound_chan[1] -> map(|(EchoMsg { payload, .. }, addr)| (EchoMsg { payload, ts: Utc::now() }, addr))
            -> map(|(m, a)| (serialize_json(m), a))
            -> dest_sink(outbound);
    };

    // run the server
    flow.run_async().await;
}
