use crate::protocol::{deserialize_msg, serialize_msg, CoordMsg, MsgType, SubordResponse};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
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
        outbound_chan = merge() -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(|m| deserialize_msg(m)) -> tee();
        votes_chan = inbound_chan[0] -> filter_map(|m: SubordResponse| match m.mtype {
                MsgType::Commit => Some(m),
                MsgType::Abort => Some(m),
                _ => None });
        p2_ack_chan = inbound_chan[1] -> filter_map(|m: SubordResponse| match m.mtype {
                MsgType::AckP2 => Some(m),
                _ => None }) -> for_each(|x| println!("Ack: {:?}", x));

        subord_total = subords[0] -> fold(0, |a,_b| a+1) -> for_each(|n| println!("There are {} subordinates.", n));

        // Phase 1 request flow:
        // Given a transaction commit request from stdio, send a Prepare Message to each subordinate
        // with the transaction ID and a unique message ID.

        // listen for commit requests -- these should be integer IDs
        // broadcast a phase 1 message upon receipt
        commits = recv_stream(stdin_lines)
            -> map(|l: Result<std::string::String, std::io::Error>| l.unwrap())
            -> filter_map(|l| match l.trim().parse::<u16>() {
                        Ok(the_xid) => {
                            Some(the_xid)
                        },
                        Err(_) => None,
                   });
        broadcast_p1_prep = join() -> tee();
        broadcast_p1_prep[0] -> for_each(|((), (xid, sub))| println!("broadcasting phase 1: {:?}, {:?}", xid, sub));
        broadcast_p1 = broadcast_p1_prep[1]
                   -> map(|((), (xid, addr))| (serialize_msg(CoordMsg{xid: xid, mid: 0, mtype: MsgType::Prepare}), addr))
                   -> [0]outbound_chan;
        commits -> map(|xid| ((), xid)) -> [0]broadcast_p1_prep;
        subords[1] -> map(|s| ((), s)) -> [1]broadcast_p1_prep;
        subords[2] -> for_each(|s| println!("Subordinate: {:?}", s));

        // count p1 votes
        // Need to do this per xact!
        votes = votes_chan -> tee();
        // total_votes = votes[0] -> fold(HashMap::new(), |mut ht, m:SubordResponse| {
        //     let e = ht.entry(m.xid).or_insert(m.xid);
        //     *e += 1;
        //     ht})
        //     -> flat_map(|ht| ht.iter())
        //     -> map(|(&xid, &c)| (xid, c)) -> tee();
        // total_votes[0] -> for_each(|(xid, c)| println!("Total votes for xid {}: {}", xid, c));
        // abort_votes = votes[1] -> filter_map(|l: SubordResponse| match l.mtype {
        //                                         MsgType::Abort => Some(l),
        //                                         _ => None
        //                                     })
        //     -> fold(HashMap::new(), |mut ht, m:SubordResponse| {
        //             let e = ht.entry(m.xid).or_insert(m.xid);
        //             *e += 1;
        //             ht})
        //     -> flat_map(|ht| ht.iter())
        //     -> map(|(&xid, &c)| (xid, c))
        //     -> for_each(|(xid,c)| println!("Total aborts for xid {}: {}", xid, c));

        // When total_votes = subord_total, it's time to send p2
        // Need to do this per xact
        // compare_votes = join() -> for_each(|m| println!("compare_votes: {:?}", m));
        // total_votes[1] -> map(|(xid, c)| (c, xid)) -> [0]compare_votes;
        // subord_total -> map(|x| (x as u16, ())) -> [1]compare_votes;

        // Send p2

        // Collect p2 responses

    };

    if let Some(graph) = opts.graph {
        match graph {
            GraphType::Mermaid => {
                println!("{}", df.generate_mermaid())
            }
            GraphType::Dot => {
                println!("{}", df.generate_dot())
            }
            GraphType::Json => {
                println!("{}", df.generate_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
