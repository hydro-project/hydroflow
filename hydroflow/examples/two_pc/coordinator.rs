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
    let (subordinates_out_tee_4_push, subordinates_out_tee_4_pull) =
        hf.make_edge::<_, VecHandoff<String>, _>("subordinates_out_tee_4");

    // create a flow to populate the subordinate handoffs
    hf.add_subgraph_stratified(
        "fetch subordinates",
        0,
        subordinates_out.flatten().pull_to_push().map(Some).tee(
            subordinates_out_tee_1_push,
            hf.start_tee().tee(
                subordinates_out_tee_2_push,
                hf.start_tee()
                    .tee(subordinates_out_tee_3_push, subordinates_out_tee_4_push),
            ),
        ),
    );

    let (subordinate_count_1_push, subordinate_count_1_pull) =
        hf.make_edge::<_, VecHandoff<u32>, _>("subordinate_count_1");
    let (subordinate_max_count_tee_1_push, subordinate_max_count_tee_1_pull) =
        hf.make_edge::<_, VecHandoff<u32>, _>("subordinate_max_count_tee_1");
    let (subordinate_max_count_tee_2_push, subordinate_max_count_tee_2_pull) =
        hf.make_edge::<_, VecHandoff<u32>, _>("subordinate_max_count_tee_2");

    // Demultiplex received messages into phase1 and phase2 handoffs
    let (p1_recv_push, p1_recv_pull) = hf.make_edge::<_, VecHandoff<SubordResponse>, _>("p1");
    let (p2_recv_push, p2_recv_pull) = hf.make_edge::<_, VecHandoff<SubordResponse>, _>("p2");

    // Create a flow to partition the received messages and populate the handoffs
    hf.add_subgraph_stratified(
        "demux",
        1,
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
    hf.add_subgraph_stratified(
        "mux",
        1,
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
    hf.add_subgraph_stratified(
        "phase 1 init",
        1,
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

    // Count the size of subordinates.
    hf.add_subgraph_stratified(
        "count the size of subordinates",
        0,
        subordinates_out_tee_4_pull
            .flatten()
            .map_scan(0u32, |counter, _| {
                *counter += 1;
                *counter
            })
            .pull_to_push()
            .map(Some)
            .push_to(subordinate_count_1_push),
    );

    // Take the max of subordinate counts. This is because subordinate_count_1_pull (which is
    // populated in the previous stratum) has a running count of messages. Thus, the maximum
    // of these counts must be computed here, in the next stratum, once all the running counts
    // have been generated.
    //
    // This is very inefficient unless there is a lot of batching since the max count is
    // recomputed every tick even though it is a constant that's set in the config file.
    hf.add_subgraph_stratified(
        "max subordinate counts",
        1,
        subordinate_count_1_pull
            .filter_map(|batch| batch.into_iter().max())
            .pull_to_push()
            .map(Some)
            .tee(
                subordinate_max_count_tee_1_push,
                subordinate_max_count_tee_2_push,
            ),
    );

    // Phase 1 Response and Phase 2 Request:
    // Receive and count subordinate responses until we have them all.
    // Once we have them all, assess the unanimous vote condition for commit and
    // send either a Commit or Abort message.
    // HACK: We move the vote_ctr((xid, mid), count) HashMap into a map operator as flow state.
    // Should be done with a groupby.
    hf.add_subgraph_stratified(
        "collect votes and send command",
        1,
        p1_recv_pull
            .flatten()
            .map(move |msg| {
                println!("{:?}", msg);
                msg
            })
            .map_scan(
                HashMap::<u16, (u32, SubordResponse, bool)>::new(),
                |ballot_counts, msg| {
                    // Count up the ballots for transactions, partitioned by transaction id.
                    // Also check for aborts.
                    let commit = msg.mtype == MsgType::Commit;
                    let v = ballot_counts.entry(msg.xid).or_insert((0, msg, commit));
                    v.0 += 1;
                    v.2 = v.2 && commit;
                    v.clone()
                },
            )
            .cross_join(subordinate_max_count_tee_2_pull.flatten())
            .filter_map(move |((recv_count, msg, commit), count)| {
                if recv_count == count {
                    // Abort if any subordinate voted to Abort
                    Some(CoordMsg {
                        xid: msg.xid,
                        mid: msg.mid + 1,
                        // Abort if any subordinate voted to Abort
                        mtype: if commit {
                            MsgType::Commit
                        } else {
                            MsgType::Abort
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
    hf.add_subgraph_stratified(
        "collect acks and end",
        1,
        p2_recv_pull
            .flatten()
            .map(|msg| {
                println!("{:?}", msg);
                msg
            })
            .map_scan(
                HashMap::<u16, (u32, SubordResponse)>::new(),
                |ack_counts, msg| {
                    // Count up the acks for transactions, partitioned by transaction id.
                    let v = ack_counts.entry(msg.xid).or_insert((0, msg));
                    v.0 += 1;
                    v.clone()
                },
            )
            .cross_join(subordinate_max_count_tee_1_pull.flatten())
            .filter_map(
                move |((recv_count, msg), count)| {
                    if recv_count == count {
                        Some(msg)
                    } else {
                        None
                    }
                },
            )
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
