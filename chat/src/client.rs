use crate::{Encode,Opts};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::{handoff::VecHandoff, net::Message};
use hydroflow::tokio::net::{TcpStream, TcpListener};

pub(crate) async fn run_client(opts: Opts) {
    let mut df = HydroflowBuilder::default();

    let stream = TcpStream::connect(format!("localhost:{}", opts.port))
        .await
        .unwrap();
    let network_out = df.add_write_tcp_stream(stream);

    let (text_in, text_out) = df.add_channel_input::<Option<_>, VecHandoff<(String, String)>>();
    hydroflow::tokio::spawn(async move {
        // TODO(mingwei): temp, need to integrate stream into surface API.
        use hydroflow::tokio::io::AsyncBufReadExt;

        let mut reader = hydroflow::tokio::io::BufReader::new(hydroflow::tokio::io::stdin());
        let mut buf = String::new();
        while let Ok(_num_chars) = reader.read_line(&mut buf).await {
            text_in.give(Some(("localhost:1111".to_owned(), buf.clone())));
            text_in.flush();
            buf.clear(); // TODO(mingwei): Maybe not needed?
        }
    });

    let sg = text_out
        .map(|vec_deque| {
            let mut buf = Vec::new();
            vec_deque.encode(&mut buf);
            Some(Message {
                address: 0,
                batch: buf.into(),
            })
        })
        .pivot()
        .reverse(network_out);
    df.add_subgraph(sg);

    let mut df = df.build();
    df.run_async().await.unwrap();
}
