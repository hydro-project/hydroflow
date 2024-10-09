use std::fmt::Debug;
use std::future::ready;
use std::hash::Hash;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::Parser;
use gossip_protocol::membership::{MemberDataBuilder, Protocol};
use gossip_protocol::{ClientRequest, GossipMessage};
use hydroflow::futures::{SinkExt, StreamExt};
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use hydroflow::{bincode, tokio};
use tracing::{error, info, trace};

use crate::config::{setup_settings_watch, SeedNodeSettings};
use crate::membership::member_name;
use crate::server::{server, SeedNode};

mod config;
mod lattices;
mod membership;
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

fn make_seed_node(settings: &SeedNodeSettings) -> SeedNode<SocketAddr> {
    SeedNode {
        id: settings.id.clone(),
        address: ipv4_resolve(&settings.address).unwrap(),
    }
}

#[hydroflow::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();

    // Setup protocol information in the member metadata.
    let client_protocol_address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), opts.client_port);
    let gossip_protocol_address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), opts.gossip_port);

    let member_data = MemberDataBuilder::new(member_name().clone())
        .add_protocol(Protocol::new("gossip".into(), gossip_protocol_address))
        .add_protocol(Protocol::new("client".into(), client_protocol_address))
        .build();

    let (client_outbound, client_inbound, _) = bind_udp_bytes(client_protocol_address).await;
    let (gossip_outbound, gossip_inbound, _) = bind_udp_bytes(gossip_protocol_address).await;

    info!(
        "Server {:?} listening for client requests on: {:?}",
        member_data.id, client_protocol_address
    );

    // TODO: Remove code duplication here.
    // Setup message serialization for outbound client responses.
    let client_ob = client_outbound.with(|(msg, addr)| {
        ready(Ok::<(hydroflow::bytes::Bytes, SocketAddr), Error>((
            hydroflow::util::serialize_to_bytes(msg),
            addr,
        )))
    });

    // Setup message deserialization for inbound client requests.
    let client_ib = client_inbound.filter_map(|input| {
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

    // Setup message serialization for outbound client responses.
    let gossip_ob = gossip_outbound.with(|(msg, addr)| {
        ready(Ok::<(hydroflow::bytes::Bytes, SocketAddr), Error>((
            hydroflow::util::serialize_to_bytes(msg),
            addr,
        )))
    });

    // Setup message deserialization for inbound client requests.
    let gossip_ib = gossip_inbound.filter_map(|input| {
        let mapped = match input {
            Ok((bytes, addr)) => {
                let msg: bincode::Result<GossipMessage> =
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

    let gossip_rx =
        IntervalStream::new(tokio::time::interval(tokio::time::Duration::from_secs(5))).map(|_| ());

    let (_watcher, server_settings, settings_stream) = setup_settings_watch();

    let seed_nodes = server_settings
        .seed_nodes
        .iter()
        .map(make_seed_node)
        .collect::<Vec<_>>();

    let seed_node_stream = settings_stream.map(|settings| {
        trace!("Settings updated. Reloading seed nodes");
        settings.seed_nodes.iter().map(make_seed_node).collect()
    });

    // Create and run the server
    let mut server = server(
        client_ib,
        client_ob,
        gossip_ib,
        gossip_ob,
        gossip_rx,
        member_data,
        seed_nodes,
        seed_node_stream,
    );

    server.run_async().await;
}
