use crate::{Decode, Encode, Opts};

use hydroflow::builder::prelude::*;
use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::{ctx::RecvCtx, graph::Hydroflow, handoff::VecHandoff, net::Message};
use hydroflow::tokio::net::{TcpListener, TcpStream};
use hydroflow::{
    compiled::{pull::SymmetricHashJoin, InputBuild, IteratorToPusherator, PusheratorBuild},
    scheduled::graph_ext::GraphExt,
    tl, tt,
};

pub(crate) async fn run_server(opts: Opts) {
    let mut hf = HydroflowBuilder::default();


    let (members_in, members_out) =
        hf.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();

    let members_out = members_out.flat_map(std::convert::identity);

    let stream = TcpListener::bind(format!("localhost:{}", opts.port))
        .await
        .unwrap();
    let (stream, _) = stream.accept().await.unwrap();

    // inbound msgs stuff
    let msgs_out = hf.add_read_tcp_stream(stream);

    let sg = msgs_out
        .flat_map(std::convert::identity)
        .flat_map(|message| <Vec<(String, String)>>::decode(message.batch).into_iter())
        .ripple_join(members_out)
        .pivot()
        .for_each(|x| {
            println!("{:?}", x);
        });
    hf.add_subgraph(sg);

    let mut hf = hf.build();

    members_in.give(Some(("localhost:1234".to_string(), "Joe".to_string())));
    members_in.give(Some(("localhost:1235".to_string(), "Mingwei".to_string())));
    members_in.flush();

    hf.run_async().await.unwrap();
}
