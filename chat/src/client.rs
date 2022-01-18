use crate::{Encode,Opts};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::{handoff::VecHandoff, net::Message};
use hydroflow::tokio::net::{TcpStream, TcpListener};

pub(crate) async fn run_client(opts: Opts) {
    let mut connect_df = HydroflowBuilder::default(); 
    let mut df = HydroflowBuilder::default(); 

    let (connect_result_port, connect_recv) = df.hydroflow.inbound_tcp_vertex::<String>().await;
    let connect_recv = df.wrap_input(connect_recv);

    let (messages_port, messages_recv) = df.hydroflow.inbound_tcp_vertex::<String>().await;
    let messages_recv = df.wrap_input(messages_recv);

    let connect_out = connect_df.hydroflow.outbound_tcp_vertex::<String>().await;
    let connect_out = connect_df.wrap_output(connect_out);

    let (input, port) = connect_df.hydroflow.add_input::<Option<(String, String, String)>, VecHandoff<(String, String, String)>>();
    let port = connect_df.wrap_input(port);


    let addr = format!("localhost:{}", opts.port);
    let my_connect_addr = format!("localhost:{}", connect_result_port);
    let my_messages_addr = format!("localhost:{}", messages_port);

    input.give(Some((addr, my_connect_addr, my_messages_addr)));
    
    connect_df.add_subgraph(
        port.pivot().reverse(connect_out)
    );

    connect_df.build().tick();
    

    let network_out = df.hydroflow.outbound_tcp_vertex().await;
    let network_out = df.wrap_output(network_out);

    // let stream = TcpStream::connect(format!("localhost:{}", opts.port))
    //     .await
    //     .unwrap();
    // let network_out = df.add_write_tcp_stream(stream);

    let (text_in, text_out) = df.add_channel_input::<Option<_>, VecHandoff<String>>();

    let sg = connect_recv.flat_map(std::convert::identity)
        .cross_join(text_out.flat_map(std::convert::identity))
        .pivot()
        .map(Some)
        .reverse(network_out);
    df.add_subgraph(sg);

    let mut df = df.build();


    

    hydroflow::tokio::spawn(async move {
        // TODO(mingwei): temp, need to integrate stream into surface API.
        use hydroflow::tokio::io::AsyncBufReadExt;

        let mut reader = hydroflow::tokio::io::BufReader::new(hydroflow::tokio::io::stdin());
        let mut buf = String::new();
        while let Ok(_num_chars) = reader.read_line(&mut buf).await {
            text_in.give(Some(buf.clone()));
            text_in.flush();
            buf.clear(); // TODO(mingwei): Maybe not needed?
        }
    });

    df.run_async().await.unwrap();
}
