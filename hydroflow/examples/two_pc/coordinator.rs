use std::collections::HashMap;
use std::collections::HashSet;

use crate::Opts;

use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

pub(crate) async fn run_coordinator(opts: Opts, subordinates: Vec<String>) {
    let mut hf = HydroflowBuilder::default();

    // We provide a command line for users to type a Transaction ID (integer) to commit.
    // setup stdio input handler
    let text_out = {
        use futures::stream::StreamExt;
        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(tokio::io::stdin());
        let lines = tokio_stream::wrappers::LinesStream::new(reader.lines())
            .map(|result| Some(result.expect("Failed to read stdin as UTF-8.")));
        hf.add_input_from_stream::<_, _, VecHandoff<String>, _>("stdin text", lines)
    };

    // setup message send/recv ports
    let msg_recv = hf
        .hydroflow
        .inbound_tcp_vertex_port::<SubordResponse>(opts.port)
        .await;
    let msg_recv = hf.wrap_input(msg_recv);
    let msg_send = hf.hydroflow.outbound_tcp_vertex::<CoordMsg>().await;
    let msg_send = hf.wrap_output(msg_send);

    // setup bootstrap data: subordinates list
    let (subordinates_in, subordinates_out) =
        hf.add_channel_input::<_, Option<String>, VecHandoff<_>>("subordinates");

    // Three separate flows have to pull from subordinates; setup a handoff for each.
    let (subordinates_out_tee_1_push, subordinates_out_tee_1_pull) =
        hf.make_edge::<_, VecHandoff<String>, _>("subordinates_out_tee_1");
    let (subordinates_out_tee_2_push, subordinates_out_tee_2_pull) =
        hf.make_edge::<_, VecHandoff<String>, _>("subordinates_out_tee_2");
    let (subordinates_out_tee_3_push, subordinates_out_tee_3_pull) =
        hf.make_edge::<_, VecHandoff<String>, _>("subordinates_out_tee_3");

    // create a flow to populate the subordinate handoffs
    hf.add_subgraph(
        "fetch subordinates",
        subordinates_out.flatten().pull_to_push().map(Some).tee(
            subordinates_out_tee_1_push,
            hf.start_tee()
                .tee(subordinates_out_tee_2_push, subordinates_out_tee_3_push),
        ),
    );

    // Demultiplex received messages into phase1 and phase2 handoffs
    let (p1_recv_push, p1_recv_pull) = hf.make_edge::<_, VecHandoff<SubordResponse>, _>("p1");
    let (p2_recv_push, p2_recv_pull) = hf.make_edge::<_, VecHandoff<SubordResponse>, _>("p2");

    // Create a flow to partition the received messages and populate the handoffs
    hf.add_subgraph(
        "demux",
        msg_recv.flatten().pull_to_push().partition(
            |m| m.mtype == MsgType::Commit || m.mtype == MsgType::Abort,
            hf.start_tee().map(Some).push_to(p1_recv_push),
            hf.start_tee().map(Some).push_to(p2_recv_push),
        ),
    );

    // Multiplex messages to send
    let (p1_send_push, p1_send_pull) =
        hf.make_edge::<_, VecHandoff<(String, CoordMsg)>, _>("p1 send");
    let (p2_send_push, p2_send_pull) =
        hf.make_edge::<_, VecHandoff<(String, CoordMsg)>, _>("p2 send");
    let (p2_send_end_push, p2_send_end_pull) =
        hf.make_edge::<_, VecHandoff<(String, CoordMsg)>, _>("p2 send end");

    // Create a flow to union together the messages and populate the network with them
    hf.add_subgraph(
        "mux",
        p1_send_pull
            .chain(p2_send_pull)
            .chain(p2_send_end_pull)
            .pull_to_push()
            .push_to(msg_send),
    );

    // Phase 1 request flow:
    // Given a transaction commit request from stdio, send a Prepare Message to each subordinate
    // with the transaction ID and a unique message ID.
    // HACK: We move the xids HashSet into a map operator as flow state.
    // Should be done with a groupby.
    let mut xids = HashSet::new();
    hf.add_subgraph(
        "phase 1 init",
        text_out
            .flatten()
            .filter_map(move |xidstr| {
                match xidstr.trim().parse::<u16>() {
                    Ok(the_xid) => {
                        if xids.contains(&the_xid) {
                            println!("Transaction ID {} already used", the_xid);
                            None
                        } else {
                            xids.insert(the_xid);
                            Some(CoordMsg {
                                xid: the_xid,
                                mid: 1, // first message for this transaction
                                mtype: MsgType::Prepare,
                            })
                        }
                    }
                    Err(_) => None,
                }
            })
            .cross_join(subordinates_out_tee_1_pull.flatten())
            .map(|(msg, addr)| Some((addr, msg)))
            .pull_to_push()
            .push_to(p1_send_push),
    );

    // Phase 1 Response and Phase 2 Request:
    // Receive and count subordinate responses until we have them all.
    // Once we have them all, assess the unanimous vote condition for commit and
    // send either a Commit or Abort message.
    // HACK: We move the vote_ctr((xid, mid), count) HashMap into a map operator as flow state.
    // Should be done with a groupby.
    let mut vote_ctr = HashMap::new();
    hf.add_subgraph(
        "collect votes and send command",
        p1_recv_pull
            .flatten()
            .map(move |msg| {
                println!("{:?}", msg);
                msg
            })
            .filter_map(move |msg| {
                *vote_ctr.entry((msg.xid, msg.mtype)).or_insert(0) += 1;
                // HACK: the constant 3 here is the length of the subordinates list!
                // Should be computed on bootstrap tick and joined in here.
                if vote_ctr.get(&(msg.xid, MsgType::Commit)).unwrap_or(&0)
                    + vote_ctr.get(&(msg.xid, MsgType::Abort)).unwrap_or(&0)
                    == 3
                {
                    // Abort if any subordinate voted to Abort
                    Some(CoordMsg {
                        xid: msg.xid,
                        mid: msg.mid + 1,
                        // Abort if any subordinate voted to Abort
                        mtype: if vote_ctr.get(&(msg.xid, MsgType::Abort)).unwrap_or(&0) > &0 {
                            MsgType::Abort
                        } else {
                            MsgType::Commit
                        },
                    })
                } else {
                    None
                }
            })
            .map(|msg| {
                println!(
                    "All votes in for xid {:?}, sending {:?}",
                    msg.xid, msg.mtype
                );
                println!("Logging {:?}", msg);
                msg
            })
            .cross_join(subordinates_out_tee_2_pull.flatten())
            .map(|(msg, addr)| Some((addr, msg)))
            .pull_to_push()
            .push_to(p2_send_push),
    );

    // Phase 2 Response: collect subordinate acks until we have them all
    // Then write to local log and send End message to all subordinates.
    let mut ack_ctr = HashMap::new();
    hf.add_subgraph(
        "collect acks and end",
        p2_recv_pull
            .flatten()
            .map(|msg| {
                println!("{:?}", msg);
                msg
            })
            .filter_map(move |msg| {
                *ack_ctr.entry((msg.xid, msg.mtype)).or_insert(0) += 1;
                // HACK
                if ack_ctr.get(&(msg.xid, MsgType::AckP2)).unwrap_or(&0) + 0 == 3 {
                    Some(msg)
                } else {
                    None
                }
            })
            .map(|smsg| {
                let cmsg = CoordMsg {
                    xid: smsg.xid,
                    mid: smsg.mid + 1,
                    mtype: MsgType::End,
                };
                println!("Logging {:?}", cmsg);
                cmsg
            })
            .cross_join(subordinates_out_tee_3_pull.flatten())
            .map(|(msg, addr)| Some((addr, msg)))
            .pull_to_push()
            .push_to(p2_send_end_push),
    );

    // start the data flowing.
    let mut hf = hf.build();
    // first populate the static subordinates into the flow
    subordinates
        .into_iter()
        .map(Some)
        .for_each(|x| subordinates_in.give(x));
    subordinates_in.flush();
    hf.run_async().await.unwrap();
}
