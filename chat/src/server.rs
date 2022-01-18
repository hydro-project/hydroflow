use crate::{Decode, Opts};

use hydroflow::scheduled::{handoff::VecHandoff};
use hydroflow::builder::prelude::*;
use hydroflow::tokio::net::TcpListener;


pub(crate) async fn run_server(opts: Opts) {
    let mut hf = HydroflowBuilder::default();

    let members_in = hf.hydroflow.inbound_tcp_vertex_port::<(String,String)>(opts.port).await;
    let members_in = hf.wrap_input(members_in);
    println!("Listening for member joins on {}", opts.port);
    
    let (port, msgs_in) = hf.hydroflow.inbound_tcp_vertex::<String>().await;
    let msgs_in = hf.wrap_input(msgs_in);
    println!("Listening for messages on {}", port);

    let (members_in, members_out) = 
        hf.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();

    let members_out = members_out.flat_map(std::convert::identity);
    let msgs_in = msgs_in.flat_map(std::convert::identity);

    let sg = members_out.cross_join(msgs_in)
                        .pivot()
                        .for_each(|x| {
       println!("{:?}", x); 
    });

    hf.add_subgraph(sg);

    // TODO: integrate these tcp vertices into builder
    // let (port, messages) = hf.wrap_input(hf.hydroflow.inbound_tcp_vertex().await);
    // let outbound_messages = hf.wrap_output(hf.hydroflow.outbound_tcp_vertex().await);
    
    // -> (ip:port, )

    let mut hf = hf.build();

    members_in.give(Some(("localhost:1234".to_string(), "Joe".to_string())));
    members_in.give(Some(("localhost:1235".to_string(), "Mingwei".to_string())));
    members_in.flush();

    hf.run_async().await.unwrap();
}
