use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use gossip_protocol::membership::MemberData;
use gossip_protocol::{ClientRequest, ClientResponse, Key, Namespace};
use hydroflow::futures::{Sink, Stream};
use hydroflow::hydroflow_syntax;
use hydroflow::itertools::Itertools;
use hydroflow::lattices::map_union::{KeyedBimorphism, MapUnionHashMap};
use hydroflow::lattices::set_union::SetUnionHashSet;
use hydroflow::lattices::PairBimorphism;
use hydroflow::scheduled::graph::Hydroflow;
use serde::Serialize;
use tracing::{info, trace};

use crate::model::{delete_row, upsert_row, Clock, Namespaces, RowKey, TableName};
use crate::util::ClientRequestWithAddress;

/// Creates a L0 key-value store server using Hydroflow.
///
/// # Arguments
/// -- `ib`: The input stream of client requests for the client protocol.
/// -- `ob`: The output sink of client responses for the client protocol.
/// -- `member_info`: The membership information of the server.
///
/// # Generic Arguments
/// -- `I`: The input stream type for the client protocol.
/// -- `O`: The output sink type for the client protocol.
/// -- `A`: The transport (address type) for the client protocol.
pub fn server<I, O, A, E>(ib: I, ob: O, member_info: MemberData<A>) -> Hydroflow<'static>
where
    I: Stream<Item = (ClientRequest, A)> + Unpin + 'static,
    O: Sink<(ClientResponse, A), Error = E> + Unpin + 'static,
    A: Hash + Debug + Clone + Eq + Serialize + 'static, // "Address"
    E: Debug + 'static,
{
    hydroflow_syntax! {

        on_start = initialize() -> tee();

        on_start -> for_each(|_| info!("Transducer started."));

        // Setup member metadata for this process.
        on_start -> map(|_| upsert_row(Clock::new(0), Namespace::System, "members".to_string(), member_info.name.clone(), serde_json::to_string(&member_info).unwrap()))
            -> namespaces;

        outbound_messages =
            inspect(|(resp, addr)| trace!("Sending response: {:?} to {:?}", resp, addr))
            -> dest_sink(ob);

        inbound_messages = source_stream(ib)
            -> map(|(msg, addr)| ClientRequestWithAddress::from_request_and_address(msg, addr))
            -> demux_enum::<ClientRequestWithAddress<A>>();

        inbound_messages[Get]
            -> inspect(|req| trace!("Received Get request: {:?} at {:?}", req, context.current_tick()))
            -> map(|(key, addr) : (Key, A)| {
                let row = MapUnionHashMap::new_from([
                        (
                            key.row_key,
                            SetUnionHashSet::new_from([addr /* to respond with the result later*/])
                        ),
                ]);
                let table = MapUnionHashMap::new_from([(key.table, row)]);
                MapUnionHashMap::new_from([(key.namespace, table)])
            })
            -> gets;

        inbound_messages[Set]
            -> inspect(|request| trace!("Received Set request: {:?} at {:?}", request, context.current_tick()))
            -> map(|(key, value, _addr) : (Key, String, A)| upsert_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key, value))
            -> namespaces;

        inbound_messages[Delete]
            -> inspect(|req| trace!("Received Delete request: {:?} at {:?}", req, context.current_tick()))
            -> map(|(key, _addr) : (Key, A)| delete_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key))
            -> namespaces;

        namespaces = union()
            -> state::<'static, Namespaces::<Clock>>();

        gets = state::<'tick, MapUnionHashMap<Namespace, MapUnionHashMap<TableName, MapUnionHashMap<RowKey, SetUnionHashSet<A>>>>>();

        namespaces -> [0]process_system_table_gets;
        gets -> [1]process_system_table_gets;

        process_system_table_gets = lattice_bimorphism(KeyedBimorphism::<HashMap<_, _>, _>::new(KeyedBimorphism::<HashMap<_, _>, _>::new(KeyedBimorphism::<HashMap<_, _>, _>::new(PairBimorphism))), #namespaces, #gets)
            -> flat_map(|result| {

                let mut response: Vec<(ClientResponse, A)> = vec![];

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
            }) -> outbound_messages;

        // Uncomment to aid with debugging.
        // source_interval(Duration::from_secs(3))
        //    -> for_each(|_| println!("State: {:?}", #namespaces));
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
    }

    /// Create a test instance of the L0 key-value store server.
    ///
    /// Arguments:
    /// -- `member_name`: The name of the member.
    /// -- `client_endpoint`: The endpoint for the client protocol.
    fn test_server<M, E>(member_name: M, client_endpoint: E) -> TestServer
    where
        M: Into<String>,
        E: Into<TestAddr>,
    {
        let (client_request_tx, client_request_rx) =
            hydroflow::util::unbounded_channel::<(ClientRequest, TestAddr)>();
        let (client_response_tx, client_response_rx) =
            hydroflow::util::unbounded_channel::<(ClientResponse, TestAddr)>();

        let builder = MemberDataBuilder::new(member_name.into()).add_protocol(Protocol::new(
            client_endpoint.into(),
            "client_endpoint".to_string(),
        ));

        let member_data = builder.build();

        let server = server(
            client_request_rx,
            sink_from_fn(move |(resp, addr)| client_response_tx.send((resp, addr)).unwrap()),
            member_data,
        );

        TestServer {
            server,
            client_requests: client_request_tx,
            client_responses: client_response_rx,
        }
    }

    const TEST_SERVER_NAME: &str = "test_server";
    const TEST_CLIENT_ENDPOINT: &str = "client_endpoint";

    #[hydroflow::test]
    async fn test_member_initialization() {
        let mut test_server = test_server(TEST_SERVER_NAME, "client_endpoint");
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
        let mut test_server = test_server("test_server", "gossip_endpoint");

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
