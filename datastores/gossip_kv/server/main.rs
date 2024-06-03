use std::collections::HashMap;
use std::net::SocketAddr;

use gossip_protocol::{ClientRequest, ClientResponse, Key, Namespace};
use hydroflow::itertools::Itertools;
use hydroflow::lattices::map_union::{KeyedBimorphism, MapUnionHashMap};
use hydroflow::lattices::set_union::SetUnionHashSet;
use hydroflow::lattices::PairBimorphism;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use hydroflow::{hydroflow_syntax, tokio, DemuxEnum};

use crate::model::{delete_row, upsert_row, Clock, Namespaces, RowKey, TableName};

mod model;

#[derive(Debug, DemuxEnum)]
enum ClientRequestWithAddress {
    Get {
        key: Key,
        addr: SocketAddr,
    },
    Set {
        key: Key,
        value: String,
        addr: SocketAddr,
    },
    Delete {
        key: Key,
        addr: SocketAddr,
    },
}

impl ClientRequestWithAddress {
    fn from_request_and_address(request: ClientRequest, addr: SocketAddr) -> Self {
        match request {
            ClientRequest::Get { key } => Self::Get { key, addr },
            ClientRequest::Set { key, value } => Self::Set { key, value, addr },
            ClientRequest::Delete { key } => Self::Delete { key, addr },
        }
    }
}

#[hydroflow::main]
async fn main() {
    let address = ipv4_resolve("localhost:3000").unwrap();
    let (outbound, inbound, _) = bind_udp_bytes(address).await;

    let mut server = hydroflow_syntax! {
        outbound_messages = dest_sink_serde(outbound);

        inbound_messages = source_stream_serde(inbound)
            -> map(Result::unwrap)
            -> map(|(msg, addr)| ClientRequestWithAddress::from_request_and_address(msg, addr))
            -> demux_enum::<ClientRequestWithAddress>();

        inbound_messages[Get]
            -> inspect(|req| println!("Received Get request: {:?}", req))
            -> map(|(key, addr) : (Key, SocketAddr)| {
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
            -> inspect(|request| println!("Received Set request: {:?}", request))
            -> map(|(key, value, _addr) : (Key, String, SocketAddr)| upsert_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key, value))
            -> namespaces;

        inbound_messages[Delete]
            -> inspect(|req| println!("Received Delete request: {:?}", req))
            -> map(|(key, _addr) : (Key, SocketAddr)| delete_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key))
            -> namespaces;

        namespaces = union()
            -> state::<'static, Namespaces::<Clock>>();

        gets = state::<'tick, MapUnionHashMap<Namespace, MapUnionHashMap<TableName, MapUnionHashMap<RowKey, SetUnionHashSet<SocketAddr>>>>>();

        namespaces -> [0]process_system_table_gets;
        gets -> [1]process_system_table_gets;

        process_system_table_gets = lattice_bimorphism(KeyedBimorphism::<HashMap<_, _>, _>::new(KeyedBimorphism::<HashMap<_, _>, _>::new(KeyedBimorphism::<HashMap<_, _>, _>::new(PairBimorphism))), #namespaces, #gets)
            -> flat_map(|result| {

                let mut response: Vec<(ClientResponse, SocketAddr)> = vec![];

                // TODO: Support multiple results. https://github.com/hydro-project/hydroflow/issues/1256
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
                                let first_value = all_values.iter().find_or_first(|_| true).unwrap();

                                let all_addresses = join_results.as_reveal_ref().1.as_reveal_ref();
                                let socket_addr = all_addresses.iter().find_or_first(|_| true).unwrap();

                                response.push((
                                    ClientResponse::Get {key, value: Some(first_value.clone())},
                                    *socket_addr,
                            ));
                        }
                    }
                }

                response

            }) -> outbound_messages;

        // Uncomment to aid with debugging.
        // source_interval(Duration::from_secs(3))
        //    -> for_each(|_| println!("State: {:?}", #namespaces));
    };

    server.run_async().await;
}
