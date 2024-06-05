use std::collections::HashMap;
use std::fmt::Debug;
use std::future::ready;
use std::hash::Hash;
use std::io::Error;
use std::net::SocketAddr;

use gossip_protocol::{ClientRequest, ClientResponse, Key, Namespace};
use hydroflow::futures::{Sink, SinkExt, Stream, StreamExt};
use hydroflow::itertools::Itertools;
use hydroflow::lattices::map_union::{KeyedBimorphism, MapUnionHashMap};
use hydroflow::lattices::set_union::SetUnionHashSet;
use hydroflow::lattices::PairBimorphism;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use hydroflow::{bincode, hydroflow_syntax, tokio, DemuxEnum};

use crate::model::{delete_row, upsert_row, Clock, Namespaces, RowKey, TableName};

mod model;

#[derive(Debug, DemuxEnum)]
enum ClientRequestWithAddress<A> {
    Get { key: Key, addr: A },
    Set { key: Key, value: String, addr: A },
    Delete { key: Key, addr: A },
}

impl<A> ClientRequestWithAddress<A> {
    fn from_request_and_address(request: ClientRequest, addr: A) -> Self {
        match request {
            ClientRequest::Get { key } => Self::Get { key, addr },
            ClientRequest::Set { key, value } => Self::Set { key, value, addr },
            ClientRequest::Delete { key } => Self::Delete { key, addr },
        }
    }
}

fn server<I, O, A, E>(ib: I, ob: O) -> Hydroflow<'static>
where
    I: Stream<Item = (ClientRequest, A)> + Unpin + 'static,
    O: Sink<(ClientResponse, A), Error = E> + Unpin + 'static,
    A: Hash + Debug + Clone + Eq + 'static, // "Address"
    E: Debug + 'static,
{
    hydroflow_syntax! {
        outbound_messages = inspect(|(resp, addr)| println!("Sending {:?} to {:?}", resp, addr) ) -> dest_sink(ob);

        inbound_messages = source_stream(ib)
            -> map(|(msg, addr)| ClientRequestWithAddress::from_request_and_address(msg, addr))
            -> demux_enum::<ClientRequestWithAddress<A>>();

        inbound_messages[Get]
            -> inspect(|req| println!("Received Get request: {:?} at {:?}", req, context.current_tick()))
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
            -> inspect(|request| println!("Received Set request: {:?} at {:?}", request, context.current_tick()))
            -> map(|(key, value, _addr) : (Key, String, A)| upsert_row(Clock::new(context.current_tick().0), key.namespace, key.table, key.row_key, value))
            -> namespaces;

        inbound_messages[Delete]
            -> inspect(|req| println!("Received Delete request: {:?} at {:?}", req, context.current_tick()))
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

#[hydroflow::main]
async fn main() {
    let address = ipv4_resolve("localhost:3000").unwrap();
    let (outbound, inbound, _) = bind_udp_bytes(address).await;

    let ob = outbound.with(|(msg, addr)| {
        println!("Sending message: {:?}", msg);
        ready(Ok::<(hydroflow::bytes::Bytes, SocketAddr), Error>((
            hydroflow::util::serialize_to_bytes(msg),
            addr,
        )))
    });

    let ib = inbound.filter_map(|input| {
        let mapped = match input {
            Ok((bytes, addr)) => {
                let msg: bincode::Result<ClientRequest> =
                    hydroflow::util::deserialize_from_bytes(&bytes);
                match msg {
                    Ok(msg) => Some((msg, addr)),
                    Err(e) => {
                        println!("Error deserializing message: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("Error receiving message: {:?}", e);
                None
            }
        };
        ready(mapped)
    });

    let mut server = server(ib, ob);

    server.run_async().await;
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::convert::Infallible;

    use hydroflow::futures::sink;

    use super::*;
    fn sink_from_fn<T>(mut f: impl FnMut(T)) -> impl Sink<T, Error = Infallible> {
        sink::drain().with(move |item| {
            (f)(item);
            ready(Result::<(), Infallible>::Ok(()))
        })
    }

    #[hydroflow::test]
    async fn test_multiple_values_same_tick() {
        let (server_input_tx, server_input_rx) =
            hydroflow::util::unbounded_channel::<(ClientRequest, ())>();
        let (server_output_tx, mut server_output_rx) =
            hydroflow::util::unbounded_channel::<(ClientResponse, ())>();

        let mut server = server(
            server_input_rx,
            sink_from_fn(move |(resp, _)| server_output_tx.send((resp, ())).unwrap()),
        );

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

        server_input_tx.send((set_a, ())).unwrap();
        server_input_tx.send((set_b, ())).unwrap();

        server.run_tick();

        // Read the value back.
        let get = ClientRequest::Get { key: key.clone() };
        server_input_tx.send((get, ())).unwrap();
        server.run_tick();

        let output = hydroflow::util::collect_ready_async::<Vec<_>, _>(&mut server_output_rx).await;
        assert_eq!(
            output,
            &[(
                ClientResponse::Get {
                    key,
                    value: HashSet::from([val_a, val_b])
                },
                ()
            )]
        );
    }
}
