use crate::Opts;

use crate::protocol::{ChatMessage, MemberRequest, MemberResponse};
use chrono::prelude::*;
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

pub(crate) async fn run_server(opts: Opts) {
    let mut hf = HydroflowBuilder::default();

    let members_in = hf
        .hydroflow
        .inbound_tcp_vertex_port::<MemberRequest>(opts.port)
        .await;
    let members_in = hf.wrap_input(members_in);
    println!("Listening for member joins on {}", opts.port);

    let members_out = hf.hydroflow.outbound_tcp_vertex::<MemberResponse>().await;
    let members_out = hf.wrap_output(members_out);

    let (port, msgs_in) = hf.hydroflow.inbound_tcp_vertex::<ChatMessage>().await;
    let msgs_in = hf.wrap_input(msgs_in);
    println!("Listening for messages on {}", port);

    let messages_out = hf.hydroflow.outbound_tcp_vertex::<ChatMessage>().await;
    let messages_out = hf.wrap_output(messages_out);

    // we're going to tee the output of members_in to 2 destinations.
    // 1. acknowledge with messages_port via members_out
    let membership_response = hf
        .start_tee()
        .map(move |req: MemberRequest| {
            Some((
                req.connect_addr,
                MemberResponse {
                    messages_port: port,
                },
            ))
        })
        .push_to(members_out);

    // 2. feed new members into the join
    // But first, we need a buffer to turn push into pull for cross_join.
    let (memberships_push, memberships_pull) = hf.make_edge::<VecHandoff<String>, Option<String>>();
    // and now the other start_tee
    let member_join_input = hf
        .start_tee()
        .map(|req: MemberRequest| req.messages_addr)
        .map(Some)
        .push_to(memberships_push);

    // Now assemble the prelude to the tee
    let sg = members_in
        .flatten()
        .pull_to_push()
        .tee(membership_response, member_join_input);
    hf.add_subgraph(sg);

    // And assemble the cross-join of msgs_in and members_in, flowing to members_out
    let msgs_in = msgs_in.flatten();

    let sg = memberships_pull
        .flatten()
        .cross_join(msgs_in)
        .map(|(addr, msg)| {
            Some((
                addr,
                ChatMessage {
                    nickname: msg.nickname,
                    message: msg.message,
                    ts: Utc::now(),
                },
            ))
        })
        .pull_to_push()
        .push_to(messages_out);

    hf.add_subgraph(sg);

    let mut hf = hf.build();

    hf.run_async().await.unwrap();
}
