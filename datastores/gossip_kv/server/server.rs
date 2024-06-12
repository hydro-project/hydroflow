use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use gossip_protocol::membership::{MemberData, MemberId};
use gossip_protocol::{ClientRequest, ClientResponse, GossipMessage, Key, Namespace};
use hydroflow::futures::{Sink, Stream};
use hydroflow::hydroflow_syntax;
use hydroflow::itertools::Itertools;
use hydroflow::lattices::map_union::{KeyedBimorphism, MapUnionHashMap, MapUnionSingletonMap};
use hydroflow::lattices::set_union::SetUnionHashSet;
use hydroflow::lattices::{Lattice, PairBimorphism};
use hydroflow::scheduled::graph::Hydroflow;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tracing::{info, trace};
use gossip_protocol::model::{Clock, delete_row, Namespaces, RowKey, TableName, upsert_row};
use lattices::IsTop;
use crate::lattices::BoundedSetLattice;

use crate::util::{ClientRequestWithAddress, GossipRequestWithAddress};

/// A trait that represents an abstract network address. In production, this will typically be
/// SocketAddr.
pub trait Address: Hash + Debug + Clone + Eq + Serialize {}
impl<A> Address for A where A: Hash + Debug + Clone + Eq + Serialize {}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct SeedNode<A>
where
    A: Address,
{
    pub id: MemberId,
    pub address: A,
}

#[derive(Debug, Clone, Serialize, Deserialize, Lattice)]
pub struct InfectingWrite {
    write: Namespaces<Clock>,
    members: BoundedSetLattice<MemberId, 3>,
}

pub type MessageId = String;


/// Creates a L0 key-value store server using Hydroflow.
///
/// # Arguments
/// -- `client_inputs`: The input stream of client requests for the client protocol.
/// -- `client_outputs`: The output sink of client responses for the client protocol.
/// -- `member_info`: The membership information of the server.
/// -- `seed_nodes`: A list of seed nodes that can be used to bootstrap the gossip cluster.
pub fn server<ClientInput, ClientOutput, GossipInput, GossipOutput, Addr, E>(
    client_inputs: ClientInput,
    client_outputs: ClientOutput,
    gossip_inputs: GossipInput,
    gossip_outputs: GossipOutput,
    member_info: MemberData<Addr>,
    seed_nodes: Vec<SeedNode<Addr>>,
) -> Hydroflow<'static>
where
    ClientInput: Stream<Item = (ClientRequest, Addr)> + Unpin + 'static,
    ClientOutput: Sink<(ClientResponse, Addr), Error = E> + Unpin + 'static,
    GossipInput: Stream<Item = (GossipMessage, Addr)> + Unpin + 'static,
    GossipOutput: Sink<(GossipMessage, Addr), Error = E> + Unpin + 'static,
    Addr: Address + DeserializeOwned + 'static,
    E: Debug + 'static,
{
    hydroflow_syntax! {

        on_start = initialize() -> tee();
        on_start -> for_each(|_| info!("Transducer started."));

        // Setup member metadata for this process.
        on_start -> map(|_| upsert_row(Clock::new(0), Namespace::System, "members".to_string(), member_info.id.clone(), serde_json::to_string(&member_info).unwrap()))
            -> writes;

        // Setup seed nodes.
        seed_nodes = source_iter(seed_nodes)
           -> inspect( |seed_node| info!("Adding seed node: {:?}", seed_node))
           -> persist()
           -> null();

        client_out =
            inspect(|(resp, addr)| trace!("Sending response: {:?} to {:?}", resp, addr))
            -> dest_sink(client_outputs);

        client_in = source_stream(client_inputs)
            -> map(|(msg, addr)| ClientRequestWithAddress::from_request_and_address(msg, addr))
            -> demux_enum::<ClientRequestWithAddress<Addr>>();

        client_in[Get]
            -> inspect(|req| trace!("Received Get request: {:?} at {:?}", req, context.current_tick()))
            -> map(|(key, addr) : (Key, Addr)| {
                let row = MapUnionHashMap::new_from([
                        (
                            key.row_key,
                            SetUnionHashSet::new_from([addr /* to respond with the result later*/])
                        ),
                ]);
                let table = MapUnionHashMap::new_from([(key.table, row)]);
                MapUnionHashMap::new_from([(key.namespace, table)])
            })
            -> reads;

        client_in[Set]
            -> inspect(|request| trace!("Received Set request: {:?} at {:?}", request, context.current_tick()))
            -> map(|(key, value, _addr) : (Key, String, Addr)| upsert_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key, value))
            -> writes;

        client_in[Delete]
            -> inspect(|req| trace!("Received Delete request: {:?} at {:?}", req, context.current_tick()))
            -> map(|(key, _addr) : (Key, Addr)| delete_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key))
            -> writes;

        gossip_in = source_stream(gossip_inputs)
            -> map(|(msg, addr)| GossipRequestWithAddress::from_request_and_address(msg, addr))
            -> demux_enum::<GossipRequestWithAddress<Addr>>();

        gossip_in[Gossip]
            -> inspect(|request| trace!("Received gossip request: {:?} at {:?}", request, context.current_tick()))
            -> null();

        gossip_in[Ack]
            -> inspect(|request| trace!("Received gossip ack: {:?} at {:?}", request, context.current_tick()))
            -> null();

        gossip_in[Nack]
            -> inspect(|request| trace!("Received gossip nack: {:?} at {:?}", request, context.current_tick()))
            -> null();

        gossip_out = dest_sink(gossip_outputs);

        writes = union();

        writes -> namespaces;
        // writes -> new_writes;

        namespaces = state::<'static, Namespaces::<Clock>>();
        new_writes = namespaces -> tee();

        reads = state::<'tick, MapUnionHashMap<Namespace, MapUnionHashMap<TableName, MapUnionHashMap<RowKey, SetUnionHashSet<Addr>>>>>();

        new_writes -> [0]process_system_table_reads;
        reads -> [1]process_system_table_reads;

        process_system_table_reads = lattice_bimorphism(KeyedBimorphism::<HashMap<_, _>, _>::new(KeyedBimorphism::<HashMap<_, _>, _>::new(KeyedBimorphism::<HashMap<_, _>, _>::new(PairBimorphism))), #namespaces, #reads)
            -> flat_map(|result| {

                let mut response: Vec<(ClientResponse, Addr)> = vec![];

                    let result = result.as_reveal_ref();
                    for (namespace, tables) in result.iter() {
                        for (table_name, table) in tables.as_reveal_ref().iter() {
                            for (row_key, join_results) in table.as_reveal_ref().iter() {
                                let key = Key {
                                    namespace: *namespace,
                                    table: table_name.clone(),
                                    row_key: row_key.clone(),
                                };

                                let timestamped_values = join_results.as_reveal_ref().0;
                                let all_values = timestamped_values.as_reveal_ref().1.as_reveal_ref();

                                let all_addresses = join_results.as_reveal_ref().1.as_reveal_ref();
                                let socket_addr = all_addresses.iter().find_or_first(|_| true).unwrap();

                                response.push((
                                    ClientResponse::Get {key, value: all_values.clone()},
                                    socket_addr.clone(),
                            ));
                        }
                    }
                }
                response
            }) -> client_out;

        new_writes -> for_each(|x| trace!("NEW WRITE: {:?}", x));

        // Step 1: Put the new writes in a map, with the write as the key and a SetBoundedLattice as the value.
        infecting_writes = union() -> state::<'static, MapUnionHashMap<MessageId, InfectingWrite>>();

        new_writes -> map(|write| {
            // Ideally, the write itself is the key, but writes are a hashmap and hashmaps don't
            // have a hash implementation. So we just generate a GUID identifier for the write
            // for now.
            // TODO: Switch to using the current_tick as the key - this will allow for cheap deletions through total order tracking!
            let id = uuid::Uuid::new_v4().to_string();
            MapUnionSingletonMap::new_from((id, InfectingWrite { write, members: BoundedSetLattice::new() }))
        }) -> infecting_writes;

        // TODO: Make this an external trigger for unit tests.
        gossip_trigger = source_interval(Duration::from_secs(5));

        gossip_messages = gossip_trigger
        -> flat_map( |_|
            {
                // TODO: This is being emitted as many times as there are ticks! Why?
                // TODO: Cloning the message is expensive, but let's roll with this for now.
                #infecting_writes.as_reveal_ref().clone()
            }
        )
        -> inspect(|x| trace!("Before: {:?}", x))
        -> filter(|(_id, infecting_write)| !infecting_write.members.is_top())
        -> inspect(|x| trace!("After: {:?}", x))
        -> map(|(id, infecting_write)| {
            trace!("Choosing a peer to gossip to. {:?}:{:?}", id, infecting_write);
            let peers = #namespaces.as_reveal_ref().get(&Namespace::System).unwrap().as_reveal_ref().get("members").unwrap().as_reveal_ref().clone();
            let (chosen_peer_name, chosen_peer_info) = peers.iter().choose(&mut thread_rng()).unwrap().clone();

            // TODO: Concurrent write resolution is pending for peer_info_value
            let peer_info_value = chosen_peer_info.as_reveal_ref().1.as_reveal_ref().iter().next().unwrap().clone();
            let peer_info_deserialized = serde_json::from_str::<MemberData<Addr>>(&peer_info_value).unwrap();
            let gossip_address = peer_info_deserialized.protocols.iter().find(|protocol| protocol.name == "gossip").unwrap().clone().endpoint;

            trace!("Chosen peer: {:?}:{:?}", chosen_peer_name, gossip_address);
            (id, infecting_write, gossip_address)
        })
        -> inspect(|(message_id, infecting_write, peer_gossip_address)| trace!("Sending write:\nMessageId:{:?}\nWrite:{:?}\nPeer Address:{:?}", message_id, infecting_write, peer_gossip_address))
        -> tee();

        gossip_messages
        -> map(|(message_id, infecting_write, peer_gossip_address): (String, InfectingWrite, Addr)| {
            let gossip_request = GossipMessage::Gossip {
                message_id: message_id.clone(),
                writes: infecting_write.write.clone(),
            };
            (gossip_request, peer_gossip_address)
        })
        -> gossip_out;

        gossip_messages
            -> map(|(message_id, write, peer_gossip_address) : (String, InfectingWrite, Addr)| {
                let dummy_peer_id = uuid::Uuid::new_v4().to_string();
                trace!("Infecting write: {:?}", dummy_peer_id);
                MapUnionSingletonMap::new_from((message_id, InfectingWrite { write: write.write, members: BoundedSetLattice::new_from([dummy_peer_id]) }))
            })
            //-> defer_tick()
            -> infecting_writes;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::convert::Infallible;
    use std::future::ready;

    use futures::SinkExt;
    use gossip_protocol::membership::{MemberDataBuilder, Protocol};
    use hydroflow::futures::sink;
    use hydroflow::tokio::sync::mpsc::UnboundedSender;
    use hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
    use hydroflow::{futures, tokio};

    use super::*;

    /// A mapping sink that applies a function to each item sent to the sink.
    fn sink_from_fn<T>(mut f: impl FnMut(T)) -> impl Sink<T, Error = Infallible> {
        sink::drain().with(move |item| {
            (f)(item);
            ready(Result::<(), Infallible>::Ok(()))
        })
    }

    /// Endpoints for the test transport are just strings.
    type TestAddr = String;

    /// A test instance of the L0 key-value store server.
    ///
    /// Encapsulates everything needed to interact with the server.
    pub struct TestServer {
        /// The Hydroflow server instance.
        pub server: Hydroflow<'static>,

        /// Send client requests to the server using this channel.
        pub client_requests: UnboundedSender<(ClientRequest, TestAddr)>,

        /// Receive client responses from the server using this stream.
        pub client_responses: UnboundedReceiverStream<(ClientResponse, TestAddr)>,

        pub gossip_requests: UnboundedSender<(GossipMessage, TestAddr)>,

        pub gossip_responses: UnboundedReceiverStream<(GossipMessage, TestAddr)>,
    }

    /// Create a test instance of the L0 key-value store server.
    ///
    /// Arguments:
    /// -- `member_name`: The name of the member.
    /// -- `client_endpoint`: The endpoint for the client protocol.
    fn test_server<M, E>(member_name: M, client_endpoint: E, gossip_endpoint: E) -> TestServer
    where
        M: Into<String>,
        E: Into<TestAddr>,
    {
        let (client_request_tx, client_request_rx) =
            hydroflow::util::unbounded_channel::<(ClientRequest, TestAddr)>();
        let (client_response_tx, client_response_rx) =
            hydroflow::util::unbounded_channel::<(ClientResponse, TestAddr)>();

        let (gossip_request_tx, gossip_request_rx) =
            hydroflow::util::unbounded_channel::<(GossipMessage, TestAddr)>();
        let (gossip_response_tx, gossip_response_rx) =
            hydroflow::util::unbounded_channel::<(GossipMessage, TestAddr)>();

        let builder = MemberDataBuilder::new(member_name.into())
            .add_protocol(Protocol::new(client_endpoint.into(), "client".to_string()))
            .add_protocol(Protocol::new(gossip_endpoint.into(), "gossip".to_string()));

        let member_data = builder.build();

        let server = server(
            client_request_rx,
            sink_from_fn(move |(resp, addr)| client_response_tx.send((resp, addr)).unwrap()),
            gossip_request_rx,
            sink_from_fn(move |(resp, addr)| gossip_response_tx.send((resp, addr)).unwrap()),
            member_data,
            vec![],
        );

        TestServer {
            server,
            client_requests: client_request_tx,
            client_responses: client_response_rx,
            gossip_requests: gossip_request_tx,
            gossip_responses: gossip_response_rx,
        }
    }

    const TEST_SERVER_NAME: &str = "test_server";
    const TEST_CLIENT_ENDPOINT: &str = "client_endpoint";

    #[hydroflow::test]
    async fn test_member_initialization() {
        let mut test_server = test_server(TEST_SERVER_NAME, "client_endpoint", "gossip_endpoint");
        let key = "/sys/members/test_server".parse::<Key>().unwrap();
        // TODO: The test fails if we don't run a tick before sending the get request. Why?
        test_server.server.run_tick(); // Finish initialization.

        // Check membership information.
        test_server
            .client_requests
            .send((
                ClientRequest::Get { key: key.clone() },
                "unit_test".to_string(),
            ))
            .unwrap();
        test_server.server.run_tick();

        let output =
            hydroflow::util::collect_ready_async::<Vec<_>, _>(&mut test_server.client_responses)
                .await;

        let expected_member_data = MemberDataBuilder::new(TEST_SERVER_NAME.to_string())
            .add_protocol(Protocol::new(
                TEST_CLIENT_ENDPOINT.to_string(),
                "client_endpoint".to_string(),
            ))
            .build();

        assert_eq!(
            output,
            &[(
                ClientResponse::Get {
                    key,
                    value: HashSet::from([serde_json::to_string(&expected_member_data).unwrap()])
                },
                "unit_test".to_string()
            )]
        );
    }

    #[hydroflow::test]
    async fn test_multiple_values_same_tick() {
        let mut test_server = test_server(TEST_SERVER_NAME, "gossip_endpoint", "client_endpoint");

        let key = Key {
            namespace: Namespace::System,
            table: "table".to_string(),
            row_key: "row".to_string(),
        };
        let val_a = "A".to_string();
        let val_b = "B".to_string();

        // Send multiple writes to the same key in the same tick.
        let set_a = ClientRequest::Set {
            key: key.clone(),
            value: val_a.clone(),
        };
        let set_b = ClientRequest::Set {
            key: key.clone(),
            value: val_b.clone(),
        };

        test_server
            .client_requests
            .send((set_a, "foo".to_string()))
            .unwrap();
        test_server
            .client_requests
            .send((set_b, "bar".to_string()))
            .unwrap();

        test_server.server.run_tick();

        // Read the value back.
        let get = ClientRequest::Get { key: key.clone() };
        test_server
            .client_requests
            .send((get, "TODO".to_string()))
            .unwrap();
        test_server.server.run_tick();

        let output =
            hydroflow::util::collect_ready_async::<Vec<_>, _>(&mut test_server.client_responses)
                .await;
        assert_eq!(
            output,
            &[(
                ClientResponse::Get {
                    key,
                    value: HashSet::from([val_a, val_b])
                },
                "TODO".to_string()
            )]
        );
    }
}
