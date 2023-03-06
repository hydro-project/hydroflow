use crate::protocol::{EchoMsg, VecClock};
use crate::{GraphType, Opts};
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::lang::lattice::LatticeRepr;
use hydroflow::lang::lattice::Merge;
use hydroflow::util::{UdpSink, UdpStream};
use std::collections::HashMap;
use std::net::SocketAddr;

pub(crate) async fn run_client(
    outbound: UdpSink,
    inbound: UdpStream,
    opts: Opts,
    addr: SocketAddr,
) {
    // server_addr is required for client
    let server_addr = opts.server_addr.expect("Client requires a server address");
    let bot_vc: <VecClock as LatticeRepr>::Repr = HashMap::new();

    println!("Client live!");

    let mut flow = hydroflow_syntax! {
        // Define shared inbound and outbound channels
        inbound_chan = source_stream_serde(inbound) -> tee();
        outbound_chan = // merge() ->  // commented out since we only use this once in the client template
            dest_sink_serde(outbound);

        // Print all messages for debugging purposes
        inbound_chan[print]
            -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), m, a));

        // given the inbound packet, bump the local clock and merge this in
        inbound_chan[merge] -> map(|(msg, _sender): (EchoMsg, SocketAddr)| msg.vc) -> [net]mergevc;
        mergevc = merge() -> fold::<'static> (bot_vc.clone(),
            |mut old: <VecClock as LatticeRepr>::Repr, vc: <VecClock as LatticeRepr>::Repr| {
                    let my_addr = format!("{:?}", addr);
                    let bump = HashMap::from([(my_addr.clone(), old[&my_addr] + 1)]);
                    <VecClock as Merge<VecClock>>::merge(&mut old, bump);
                    <VecClock as Merge<VecClock>>::merge(&mut old, vc);
                    old.clone()
            }
        );

        // for each input from stdin, bump the local vc and send it to the server with the (post-bump) local vc
        input = source_stdin() -> map(|l| l.unwrap()) -> tee();
        input[tick] -> map(|_| bot_vc.clone()) -> [input]mergevc;

        // stamp each input with the latest local vc (as of this tick!)
        input[send] -> [0]stamped_output;
        mergevc[useful] -> [1]stamped_output;
        stamped_output = cross_join<'tick, 'tick>() -> map(|(l, the_vc): (String, <VecClock as LatticeRepr>::Repr)| (EchoMsg { payload: l, vc: the_vc }, server_addr));

        // and send to server
        stamped_output[send] -> outbound_chan;
    };

    if let Some(graph) = opts.graph {
        let serde_graph = flow
            .serde_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    flow.run_async().await.unwrap();
}
