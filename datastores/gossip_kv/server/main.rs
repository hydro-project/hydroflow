use std::fmt::Debug;
use std::future::ready;
use std::hash::Hash;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::Parser;
use gossip_protocol::membership::{MemberDataBuilder, Protocol};
use gossip_protocol::ClientRequest;
use hydroflow::futures::{SinkExt, StreamExt};
use hydroflow::util::bind_udp_bytes;
use hydroflow::{bincode, tokio};
use tracing::{error, info};

use crate::membership::member_name;
use crate::server::{server, SeedNode};

mod config;
mod membership;
mod model;
mod server;
mod util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Parser)]
struct Opts {
    /// Port to listen for gossip messages.
    #[clap(short, long, default_value = "3000")]
    gossip_port: u16,

    /// Port to listen for client requests.
    #[clap(short, long, default_value = "3001")]
    client_port: u16,
}

#[hydroflow::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    let settings = config::ServerSettings::new().unwrap();

    // Setup protocol information in the member metadata.
    let client_protocol_address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), opts.client_port);
    let member_data = MemberDataBuilder::new(member_name().clone())
        .add_protocol(Protocol::new("client".into(), client_protocol_address))
        .build();

    let (outbound, inbound, _) = bind_udp_bytes(client_protocol_address).await;

    info!(
        "Server {:?} listening for client requests on: {:?}",
        member_data.id, client_protocol_address
    );

    // Setup message serialization for outbound client responses.
    let ob = outbound.with(|(msg, addr)| {
        ready(Ok::<(hydroflow::bytes::Bytes, SocketAddr), Error>((
            hydroflow::util::serialize_to_bytes(msg),
            addr,
        )))
    });

    // Setup message deserialization for inbound client requests.
    let ib = inbound.filter_map(|input| {
        let mapped = match input {
            Ok((bytes, addr)) => {
                let msg: bincode::Result<ClientRequest> =
                    hydroflow::util::deserialize_from_bytes(&bytes);
                match msg {
                    Ok(msg) => Some((msg, addr)),
                    Err(e) => {
                        error!("Error deserializing message: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Error receiving message: {:?}", e);
                None
            }
        };
        ready(mapped)
    });

    // Configure seed nodes for gossip protocol.
    let seed_nodes = settings
        .seed_nodes
        .iter()
        .map(|node| SeedNode {
            id: node.id.clone(),
            address: SocketAddr::new(node.ip.parse().unwrap(), node.port),
        })
        .collect();

    // Create and run the server
    let mut server = server(ib, ob, member_data, seed_nodes);
    server.run_async().await;
}
