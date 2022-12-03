use crate::helpers::{deserialize_msg, parse_out, serialize_msg};
use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use hydroflow::pusherator::Pusherator;
use hydroflow::scheduled::graph::Hydroflow;
use std::net::SocketAddr;
use tokio::io::AsyncBufReadExt;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::LinesStream;

pub(crate) async fn run_coordinator(opts: Opts, subordinates: Vec<String>) {
    // setup message send/recv ports
    let server_socket = UdpSocket::bind(("127.0.0.1", opts.port)).await.unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(server_socket);

    // We provide a command line for users to type a Transaction ID (integer) to commit.
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let stdin_lines = LinesStream::new(reader.lines());

    let mut df: Hydroflow = hydroflow_syntax! {
        // fetch subordinates from file, convert ip:port to a SocketAddr, and tee
        subords = recv_iter(subordinates)
            -> map(|s| s.parse::<SocketAddr>().unwrap())
            -> tee();

        // set up channels
        outbound_chan = tee();
        outbound_chan[0] -> map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg) -> tee();
        msgs = inbound_chan[0] ->  demux(|m:SubordResponse, tl!(commits, aborts, acks, endeds, errs)| match m.mtype {
                    MsgType::Commit => commits.give(m),
                    MsgType::Abort => aborts.give(m),
                    MsgType::AckP2 {..} => acks.give(m),
                    MsgType::Ended {..} => endeds.give(m),
                    _ => errs.give(m),
                });
        msgs[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));
        msgs[endeds] -> null();

        // we log all messages (in this prototype we just print)
        inbound_chan[1] -> for_each(|m| println!("Received {:?}", m));
        outbound_chan[1] -> for_each(|(m, a)| println!("Sending {:?} to {:?}", m, a));

        // setup broadcast channel to all subords
        broadcast_join = cross_join() -> outbound_chan;
        broadcast = merge() -> [0]broadcast_join;
        subords[1] -> [1]broadcast_join;
        subords[2] -> for_each(|s| println!("Subordinate: {:?}", s));

        // Phase 1 initiate:
        // Given a transaction commit request from stdio, broadcast a Prepare to subordinates
        recv_stream(stdin_lines)
            -> filter_map(|l: Result<std::string::String, std::io::Error>| parse_out(l.unwrap()))
            -> map(|xid| CoordMsg{xid, mtype: MsgType::Prepare})
            -> [0]broadcast;

        // Phase 1 responses:
        // as soon as we get an abort message for P1, we start Phase 2 with Abort.
        // We'll respond to each abort message: this is redundant but correct (and monotone)
        msgs[aborts]
            -> map(|m: SubordResponse| CoordMsg{xid: m.xid, mtype: MsgType::Abort})
            -> [1]broadcast;

        // count commit votes
        // persistence done explicitly here!
        commit_buf = merge();
        commit_votes = tee();
        msgs[commits]
            -> map(|m:SubordResponse| (m.xid, 1)) -> [0]commit_buf;
        commit_buf
            -> groupby(|| 0, |old: &mut u32, val: u32| *old += val)
            -> commit_votes;
        commit_votes[0] -> next_epoch() -> [1]commit_buf;

        // count subordinates
        subord_total = subords[0] -> fold(0, |a,_b| a+1); // -> for_each(|n| println!("There are {} subordinates.", n));

        // If commit_votes for this xid is the same as all_votes, send a P2 Commit message
        committed = join() -> map(|(_c, (xid, ()))| xid);
        commit_votes[1] -> map(|(xid, c)| (c, xid)) -> [0]committed;
        subord_total -> map(|c| (c, ())) -> [1]committed;
        committed -> map(|xid| CoordMsg{xid, mtype: MsgType::Commit}) -> [2]broadcast;

        // Handle p2 acknowledgments by sending an End message
        msgs[acks]  -> map(|m:SubordResponse| CoordMsg{xid: m.xid, mtype: MsgType::End,})
                    -> [3]broadcast;

        // Handler for ended acknowledgments not necessary; we just print them
    };

    if let Some(graph) = opts.graph {
        let serde_graph = df
            .serde_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
