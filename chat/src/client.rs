use chrono::prelude::*;
use colored::Colorize;

use crate::protocol::{ChatMessage, MemberRequest, MemberResponse};
use crate::Opts;
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::handoff::VecHandoff;

pub(crate) async fn run_client(opts: Opts) {
    let mut df = HydroflowBuilder::default();

    // setup connection req/resp ports
    let connect_req = df.hydroflow.outbound_tcp_vertex::<MemberRequest>().await;
    let connect_req = df.wrap_output(connect_req);
    let (connect_response_port, connect_resp) =
        df.hydroflow.inbound_tcp_vertex::<MemberResponse>().await;
    let connect_resp = df.wrap_input(connect_resp);

    // setup message send/recv ports
    let (messages_port, messages_recv) = df.hydroflow.inbound_tcp_vertex::<ChatMessage>().await;
    let messages_recv = df.wrap_input(messages_recv);
    let messages_send = df.hydroflow.outbound_tcp_vertex().await;
    let messages_send = df.wrap_output(messages_send);

    // setup stdio input handler
    let (text_in, text_out) = df.add_channel_input::<Option<_>, VecHandoff<String>>();
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

    let addr = format!("localhost:{}", opts.port);
    let connect_addr = format!("localhost:{}", connect_response_port);
    let messages_addr = format!("localhost:{}", messages_port);

    // set up the flow for requesting to be a member
    let (my_info_send, my_info_recv) = df
        .hydroflow
        .make_edge::<VecHandoff<(String, MemberRequest)>>();
    let my_info_set = df.hydroflow.add_input(my_info_send);
    let my_info_get = df.wrap_input(my_info_recv);
    my_info_set.give(Some((
        addr,
        MemberRequest {
            nickname: opts.name.clone(),
            connect_addr,
            messages_addr,
        },
    )));
    df.add_subgraph(my_info_get.pull_to_push().push_to(connect_req));

    let nickname = opts.name.clone();
    let nick2 = nickname.clone();
    // set up the flow for sending messages
    let sg = connect_resp
        .flatten()
        .cross_join(text_out.flatten())
        .pull_to_push()
        .map(move |(member_response, text)| {
            (
                format!("localhost:{}", member_response.messages_port),
                ChatMessage {
                    nickname: nickname.to_string(),
                    message: text,
                    ts: Utc::now(),
                },
            )
        })
        .map(Some)
        .push_to(messages_send);
    df.add_subgraph(sg);

    // set up the flow for receiving messages
    df.add_subgraph(
        messages_recv
            .flatten()
            .filter(move |x| x.nickname != nick2)
            .pull_to_push()
            .for_each(move |msg| {
                print!(
                    "{} {}: {}",
                    msg.ts
                        .with_timezone(&Local)
                        .format("%b %-d, %-I:%M:%S")
                        .to_string()
                        .truecolor(126, 126, 126)
                        .italic(),
                    msg.nickname.green().italic(),
                    msg.message,
                );
            }),
    );

    let mut df = df.build();
    df.run_async().await.unwrap();
}
