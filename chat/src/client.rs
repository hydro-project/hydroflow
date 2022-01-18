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
        // this boilerplate from tokio seems like a bummer to have in every program that needs kb input
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

    // set up the connection flow
    let (my_info_set, my_info_get) = df
        .hydroflow
        .add_input::<Option<(String, MemberRequest)>, VecHandoff<(String, MemberRequest)>>();
    let my_info_get = df.wrap_input(my_info_get);
    my_info_set.give(Some((
        addr,
        MemberRequest {
            nickname: opts.name.clone(),
            connect_addr,
            messages_addr,
        },
    )));
    df.add_subgraph(my_info_get.pivot().reverse(connect_req));

    // set up the message send flow
    let sg = connect_resp
        .flat_map(std::convert::identity)
        .map(|msg| {
            println!("received {:?}", msg);
            msg
        })
        .cross_join(text_out.flat_map(std::convert::identity))
        .pivot()
        .map(move |(member_response, text)| {
            (
                format!("localhost:{}", member_response.messages_port),
                ChatMessage {
                    nickname: opts.name.clone(),
                    message: text,
                },
            )
        })
        .map(Some)
        .reverse(messages_send);
    df.add_subgraph(sg);

    // set up the message receive flow
    df.add_subgraph(
        messages_recv
            .flat_map(std::convert::identity)
            .pivot()
            .for_each(|msg| {
                println!("received message {:?}", msg);
            }),
    );

    let mut df = df.build();
    df.run_async().await.unwrap();
}
