use chrono::prelude::*;
use colored::Colorize;

use crate::protocol::{ChatMessage, MemberRequest, MemberResponse};
use crate::{GraphType, Opts};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

pub(crate) async fn run_client(opts: Opts) {
    let mut hf = HydroflowBuilder::default();

    // setup connection req/resp ports
    let connect_req = hf.hydroflow.outbound_tcp_vertex::<MemberRequest>().await;
    let connect_req = hf.wrap_output(connect_req);
    let (connect_response_port, connect_resp) =
        hf.hydroflow.inbound_tcp_vertex::<MemberResponse>().await;
    let connect_resp = hf.wrap_input(connect_resp);

    // setup message send/recv ports
    let (messages_port, messages_recv) = hf.hydroflow.inbound_tcp_vertex::<ChatMessage>().await;
    let messages_recv = hf.wrap_input(messages_recv);
    let messages_send = hf.hydroflow.outbound_tcp_vertex().await;
    let messages_send = hf.wrap_output(messages_send);

    // setup stdio input handler
    let text_out = {
        use futures::stream::StreamExt;
        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(tokio::io::stdin());
        let lines = tokio_stream::wrappers::LinesStream::new(reader.lines())
            .map(|result| Some(result.expect("Failed to read stdin as UTF-8.")));
        hf.add_input_from_stream::<_, _, VecHandoff<String>, _>("stdin", lines)
    };

    // format addresses
    let addr = format!("localhost:{}", opts.port);
    let connect_addr = format!("localhost:{}", connect_response_port);
    let messages_addr = format!("localhost:{}", messages_port);

    // set up the flow for requesting to be a member
    let init_info = (
        addr,
        MemberRequest {
            nickname: opts.name.clone(),
            connect_addr,
            messages_addr,
        },
    );
    hf.add_subgraph(
        "my_info",
        std::iter::once(Some(init_info))
            .into_hydroflow()
            .pull_to_push()
            .push_to(connect_req),
    );

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
    hf.add_subgraph("sending messages", sg);

    // set up the flow for receiving messages
    hf.add_subgraph(
        "receiving messages",
        messages_recv
            .flatten()
            .filter(move |x| x.nickname != nick2)
            .pull_to_push()
            .for_each(move |msg| {
                println!(
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

    let mut hf = hf.build();
    match opts.graph {
        GraphType::Mermaid => {
            println!("{}", hf.generate_mermaid())
        }
        GraphType::Dot => {
            println!("{}", hf.generate_dot())
        }
        GraphType::JSON => {
            println!("{}", hf.generate_json())
        }
    }
    hf.run_async().await.unwrap();
}
