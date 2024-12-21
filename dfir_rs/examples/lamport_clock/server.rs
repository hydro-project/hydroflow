use std::net::SocketAddr;

use chrono::prelude::*;
use dfir_rs::dfir_syntax;
use dfir_rs::lattices::{Max, Merge};
use dfir_rs::scheduled::graph::Dfir;
use dfir_rs::util::{UdpSink, UdpStream};

use crate::protocol::EchoMsg;
use crate::Opts;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, opts: Opts) {
    let bot: Max<usize> = Max::new(0);
    println!("Server live!");

    let mut flow: Dfir = dfir_syntax! {
        // Define a shared inbound channel
        inbound_chan = source_stream_serde(inbound) -> map(Result::unwrap) -> tee();

        // Print all messages for debugging purposes
        inbound_chan[print]
            -> for_each(|(msg, addr): (EchoMsg, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), msg, addr));

        // merge in the msg vc to the local vc
        inbound_chan[merge] -> map(|(msg, _addr): (EchoMsg, SocketAddr)| msg.lamport_clock) -> mergevc;
        mergevc = fold::<'static>(
            || bot,
            |old: &mut Max<usize>, lamport_clock: Max<usize>| {
                let bump = Max::new(old.into_reveal() + 1);
                old.merge(bump);
                old.merge(lamport_clock);
            }
        );


        // Echo back the Echo messages, stamped with updated vc timestamp
        inbound_chan[1] -> map(|(EchoMsg {payload, ..}, addr)| (payload, addr) )
            -> [0]stamped_output;
        mergevc -> [1]stamped_output;
        stamped_output = cross_join::<'tick, 'tick>() -> map(|((payload, addr), the_clock): ((String, SocketAddr), Max<usize>)| (EchoMsg { payload, lamport_clock: the_clock }, addr))
            -> dest_sink_serde(outbound);
    };

    #[cfg(feature = "debugging")]
    if let Some(graph) = opts.graph {
        let serde_graph = flow
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        serde_graph.open_graph(graph, opts.write_config).unwrap();
    }
    let _ = opts;

    // run the server
    flow.run_async().await;
}
