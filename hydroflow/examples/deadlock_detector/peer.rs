use crate::helpers::{deserialize_msg, gen_bool, parse_edge, serialize_msg};
use crate::protocol::{Message, SimplePath};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use std::collections::HashMap;
use std::convert::identity;
use std::net::SocketAddr;
use tokio::io::AsyncBufReadExt;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::LinesStream;

pub(crate) async fn run_detector(opts: Opts, peer_list: Vec<String>) {
    // setup message send/recv ports
    let server_socket = UdpSocket::bind(("127.0.0.1", opts.port)).await.unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(server_socket);

    // We provide a command line for users to type waits-for edges (u32,u32).
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let stdin_lines = LinesStream::new(reader.lines());

    let mut df: Hydroflow = hydroflow_syntax! {
        // fetch peers from file, convert ip:port to a SocketAddr, and tee
        peers = recv_iter(peer_list)
            -> map(|s| s.parse::<SocketAddr>().unwrap())
            -> tee();

        // set up channels
        outbound_chan = map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg::<Message>);

        // setup gossip channel to all peers. gen_bool chooses True with the odds passed in.
        gossip_join = cross_join()
            -> filter(|_| gen_bool(0.8)) -> outbound_chan;
        gossip = map(identity) -> [0]gossip_join;
        peers[1] -> [1]gossip_join;
        peers[2] -> for_each(|s| println!("Peer: {:?}", s));

        // prompt for input
        recv_iter([()]) -> for_each(|_s| println!("Type in an edge as a tuple of two integers (x,y): "));
        // read in edges from stdin
        new_edges = recv_stream(stdin_lines)
            -> filter_map(|line| {
                parse_edge(line.unwrap())});

        // persist an edges set
        edges = merge() -> tee();
        edges[0] -> next_epoch() -> [1]edges;

        // add new edges locally
        new_edges -> [0]edges;

        // gossip all edges
        edges[1] -> fold(Message::new(), |mut m, edge| {
            m.edges.insert(edge);
            m
        }) -> gossip;


        // Form local transitive closure (from scratch) and check for cycles.
        // This is datalog TC, extended with a SimplePath vector being accumulated along the way.
        // Rule 1: add inbound edges to paths
        // paths(from, to, [from, to] :- edges(from, to))
        inbound_chan[4] -> flat_map(|m| m.edges) -> [2]edges;
        paths = merge();
        edges[2] -> map(|(from, to)| (from, to, SimplePath::new(vec![from, to]))) -> [0]paths;

        // Rule 2: form new_paths from the join of acyclic paths and edges
        // new_paths(from, to, path.append(to)) :- paths(from, mid, path), edges(mid, to), paths.cycle() == false
        new_paths = join() -> map(|(_mid, ((from, mut path), to))| {
            path.push(to);
            (from, to, path)
         }) -> tee();
        paths -> filter_map(|(from, to, path)| {
            if path.cycle() {None} // don't extend self-loops
            else {Some((to, (from, path)))}
        }) -> [0]new_paths;
        edges[3] -> map(|(from, to)| (from, to)) -> [1]new_paths;
        // stdio(from, to, path) :- new_paths(from, to, path)
        new_paths[0]
          -> filter_map(|(from, to, path)| if from == to {Some(path)} else {None})
          -> fold (HashMap::new(), |mut total, path| {
             total.insert(path.ordered(), path);
             total
          })
          -> flat_map(|m| m.into_values())
          -> for_each(|path: SimplePath<u32>| {
                println!("cycle found: {:?}", path.format());
          });
        // paths :- new_paths
        new_paths[1] -> [1]paths;

    };

    if let Some(graph) = opts.graph {
        match graph {
            GraphType::Mermaid => {
                println!(
                    "{}",
                    df.serde_graph()
                        .expect("No graph found, maybe failed to parse.")
                        .to_mermaid()
                )
            }
            GraphType::Dot => {
                println!(
                    "{}",
                    df.serde_graph()
                        .expect("No graph found, maybe failed to parse.")
                        .to_dot()
                )
            }
            GraphType::Json => {
                unimplemented!();
            }
        }
    }

    df.run_async().await.unwrap();
}
