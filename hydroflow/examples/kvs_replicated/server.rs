use std::net::SocketAddr;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::{KvsMessage, KvsMessageWithAddr};
use crate::GraphType;

pub(crate) async fn run_server(
    outbound: UdpSink,
    inbound: UdpStream,
    graph: Option<GraphType>,
    peer_server: Option<SocketAddr>,
) {
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // Network channels
        network_send = union() -> dest_sink_serde(outbound);
        network_recv = source_stream_serde(inbound)
            -> _upcast(Some(Delta))
            -> map(Result::unwrap)
            -> inspect(|(msg, addr)| println!("Message received {:?} from {:?}", msg, addr))
            -> map(|(msg, addr)| KvsMessageWithAddr::from_message(msg, addr))
            -> demux_enum::<KvsMessageWithAddr>();
        network_recv[ServerResponse] -> for_each(|(key, value, addr)| eprintln!("Unexpected server response {:?}->{:?} from {:?}", key, value, addr));
        peers = network_recv[PeerJoin] -> map(|(peer_addr,)| peer_addr) -> tee();
        network_recv[PeerGossip] -> writes;
        network_recv[ClientPut] -> writes;
        writes = union() -> tee();
        gets = network_recv[ClientGet];

        // Join as a peer if peer_server is set.
        source_iter_delta(peer_server) -> map(|peer_addr| (KvsMessage::PeerJoin, peer_addr)) -> network_send;

        // join PUTs and GETs by key
        writes -> map(|(key, value, _addr)| (key, value)) -> writes_store;
        writes_store = persist() -> tee();
        writes_store -> [0]lookup;
        gets -> [1]lookup;
        lookup = join();
        // network_send lookup responses back to the client address from the GET
        lookup[1]
            -> inspect(|tup| println!("Found a match: {:?}", tup))
            -> map(|(key, (value, client_addr))| (KvsMessage::ServerResponse { key, value }, client_addr))
            -> network_send;

        // Peers: When a new peer joins, send them all data.
        writes_store -> [0]peer_join;
        peers -> [1]peer_join;
        peer_join = cross_join()
            -> map(|((key, value), peer_addr)| (KvsMessage::PeerGossip { key, value }, peer_addr))
            -> network_send;

        // Outbound gossip. Send received PUTs to peers.
        peers -> peer_store;
        source_iter_delta(peer_server) -> peer_store;
        peer_store = union() -> persist();
        writes -> [0]outbound_gossip;
        peer_store -> [1]outbound_gossip;
        outbound_gossip = cross_join()
            -> filter(|((_key, _value, writer_addr), peer_addr)| writer_addr != peer_addr)
            -> map(|((key, value, _writer_addr), peer_addr)| (KvsMessage::PeerGossip { key, value }, peer_addr))
            -> network_send;
    };

    if let Some(graph) = graph {
        let serde_graph = df
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                serde_graph.open_mermaid().unwrap();
            }
            GraphType::Dot => {
                serde_graph.open_dot().unwrap();
            }
            GraphType::Json => {
                unimplemented!();
            }
        }
    }

    df.run_async().await.unwrap();
}
