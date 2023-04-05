use crate::protocol::{EchoMsg, VecClock};
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::lang::lattice::{LatticeRepr, Merge};
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};
use std::collections::HashMap;
use std::net::SocketAddr;

pub(crate) async fn run_server(outbound: UdpSink, inbound: UdpStream, opts: crate::Opts) {
    let bot_vc: <VecClock as LatticeRepr>::Repr = HashMap::new();
    println!("Server live!");

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Define a shared inbound channel
        inbound_chan = source_stream_serde(inbound) -> map(Result::unwrap) -> tee();

        // Print all messages for debugging purposes
        inbound_chan[print]
            -> for_each(|(msg, addr): (EchoMsg, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), msg, addr));

        // merge in the msg vc to the local vc
        inbound_chan[merge] -> map(|(msg, _addr): (EchoMsg, SocketAddr)| msg.vc) -> mergevc;
        mergevc = fold::<'static> (bot_vc,
                    |mut old: <VecClock as LatticeRepr>::Repr, vc: <VecClock as LatticeRepr>::Repr| {
                            let my_addr = format!("{:?}", opts.addr.unwrap());
                            let bump = HashMap::from([(my_addr.clone(), old[&my_addr] + 1)]);
                            <VecClock as Merge<VecClock>>::merge(&mut old, bump);
                            <VecClock as Merge<VecClock>>::merge(&mut old, vc);
                            old.clone()
                    }
                );


        // Echo back the Echo messages, stamped with updated vc timestamp
        inbound_chan[1] -> map(|(EchoMsg {payload, ..}, addr)| (payload, addr) )
            -> [0]stamped_output;
        mergevc -> [1]stamped_output;
        stamped_output = cross_join::<'tick, 'tick>() -> map(|((payload, addr), vc): ((String, SocketAddr), <VecClock as LatticeRepr>::Repr)| (EchoMsg { payload, vc }, addr))
            -> dest_sink_serde(outbound);
    };

    // run the server
    flow.run_async().await;
}
