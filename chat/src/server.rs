use crate::{Decode, Encode, Opts};

use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::{ctx::RecvCtx, graph::Hydroflow, handoff::VecHandoff, net::Message};
use hydroflow::builder::prelude::*;
use hydroflow::tokio::net::TcpStream;
use hydroflow::{
    compiled::{pull::SymmetricHashJoin, InputBuild, IteratorToPusherator, PusheratorBuild},
    scheduled::graph_ext::GraphExt,
    tl, tt,
};

pub(crate) async fn run_server(opts: Opts) {
    let mut hf = HydroflowBuilder::default();

    // let members:Vec<(String, String)>; // (id, nickname) pair
    // let msgs:Vec:String; 

    let (members_in, members_out) = hf.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();
    let (msgs, msgs_out) = hf.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();

    // let members_out = members_out.flat_map(std::convert::identity).map(|x| ((), x));
    // let msgs_out = msgs_out.flat_map(std::convert::identity).map(|x| ((), x));

    // let sg = members_out.join(msgs_out).pivot().for_each(|x| {
    //    println!("{:?}", x); 
    // });

    let members_out = members_out.flat_map(std::convert::identity);
    let msgs_out = msgs_out.flat_map(std::convert::identity);

    let sg = members_out.ripple_join(msgs_out).pivot().for_each(|x| {
       println!("{:?}", x); 
    });

    hf.add_subgraph(sg);

    {
        let mut hf = hf.build();

        members_in.give(Some(("localhost:1234".to_string(), "Joe".to_string())));
        members_in.give(Some(("localhost:1235".to_string(), "Mingwei".to_string())));
        members_in.flush();
        hf.tick();

        msgs.give(Some(("localhost:1234".to_string(), "Do you hear me?".to_string())));
        msgs.flush();
        hf.tick();

        msgs.give(Some(("localhost:1235".to_string(), "Ack!".to_string())));
        msgs.flush();
        hf.tick();
    }
}
