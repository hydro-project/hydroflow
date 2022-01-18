use crate::Opts;

use crate::protocol::{ChatMessage, MemberRequest, MemberResponse};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

pub(crate) async fn run_server(opts: Opts) {
    let mut hf = HydroflowBuilder::default();

    let members_in = hf
        .hydroflow
        .inbound_tcp_vertex_port::<MemberRequest>(opts.port)
        .await; // it's a bummer that I have to say "await", seems too low level and Tokio-ish
    let members_in = hf.wrap_input(members_in); // this is bc we don't tcp_vertex in builder
    println!("Listening for member joins on {}", opts.port);

    let members_out = hf.hydroflow.outbound_tcp_vertex::<MemberResponse>().await;
    let members_out = hf.wrap_output(members_out);

    let (port, msgs_in) = hf.hydroflow.inbound_tcp_vertex::<ChatMessage>().await;
    let msgs_in = hf.wrap_input(msgs_in);
    println!("Listening for messages on {}", port);

    let messages_out = hf.hydroflow.outbound_tcp_vertex::<ChatMessage>().await;
    let messages_out = hf.wrap_output(messages_out);

    // we need a buffer to turn push into pull for cross_join... could alternately implement a push cross_join
    let (memberships_push, memberships_pull) =
        hf.make_handoff::<VecHandoff<String>, Option<String>>();

    // tee is sensible flow-oriented thinking.
    // but for coders it may be nicer to have variable names and reuse.
    let sg = members_in.flat_map(std::convert::identity).pivot().tee(
        // Need two branches!
        // 1. Send the response back to the client.
        hf.start_tee() // start_tee feels awkward; compare to closure or variable reuse
            .map(move |req: MemberRequest| {
                Some((
                    req.connect_addr,
                    MemberResponse {
                        messages_port: port,
                    },
                ))
            })
            .reverse(members_out),
        // 2. Send the results over to the cross join.
        hf.start_tee()
            .map(|req: MemberRequest| req.messages_addr)
            .map(Some)
            .reverse(memberships_push), // reverse is a weird name for whatever you mean
    );
    hf.add_subgraph(sg);

    let msgs_in = msgs_in.flat_map(std::convert::identity);

    let sg = memberships_pull
        .flat_map(std::convert::identity) // why do I have to keep flat_mapping everywhere?
        .cross_join(msgs_in)
        .map(|(addr, msg)| {
            Some((
                addr,
                ChatMessage {
                    nickname: msg.nickname,
                    message: msg.message,
                },
            ))
        })
        .pivot() // I hate having to think about pivot; at best needs a different name
        .reverse(messages_out);

    hf.add_subgraph(sg);

    let mut hf = hf.build();

    hf.run_async().await.unwrap();
}
