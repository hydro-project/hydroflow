use crate::{Decode, Opts};

use hydroflow::scheduled::{handoff::VecHandoff};
use hydroflow::builder::prelude::*;
use hydroflow::tokio::net::TcpListener;


pub(crate) async fn run_server(opts: Opts) {
    // let stream = TcpListener::bind(format!("localhost:{}", opts.port))
    // .await
    // .unwrap();
    // let (stream, _) = stream.accept().await.unwrap();

    let mut hf = HydroflowBuilder::default();

    let stream = TcpListener::bind(format!("localhost:{}", opts.port))
        .await
        .unwrap();
    let (stream, _) = stream.accept().await.unwrap();

    // inbound msgs stuff
    let msgs_out = hf.add_read_tcp_stream(stream);
    
    let (members_in, members_out) = hf.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();
    // let (msgs, msgs_out) = hf.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();

    let members_out = members_out.flat_map(std::convert::identity);
    let msgs_out = 
        msgs_out
        .flat_map(std::convert::identity)
        .flat_map(|message| <Vec<(String, String)>>::decode(message.batch).into_iter());
    // let msgs_out = msgs_out.flat_map(std::convert::identity);

    let sg = members_out.cross_join(msgs_out)
                                                      .pivot()
                                                      .for_each(|x| {
       println!("{:?}", x); 
    });

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
