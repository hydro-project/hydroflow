use super::KVSRequest;
use super::KVSResponse;
use hydroflow::util::deserialize_from_bytes;
use hydroflow::util::serialize_to_bytes;
use hydroflow::{hydroflow_syntax, util::tcp_bytes};
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn run_server(server_addr: SocketAddr) {
    println!("tid server: {}", palaver::thread::gettid());

    let stream = TcpListener::bind(server_addr).await.unwrap();

    let (stream, _) = stream.accept().await.unwrap();
    stream.set_nodelay(true).unwrap();

    println!("accepted");

    let (outbound, inbound) = tcp_bytes(stream);

    let mut df = hydroflow_syntax! {
        // outbound_chan = dest_sink_serde_tcp(outbound);
        deserialize_input = filter_map(|payload| {
            if let Ok(payload) = payload {
                Some(deserialize_from_bytes(payload))
            } else {
                None
            }
        });

        serialize_output = map(|x| serialize_to_bytes(x));

        my_demux = source_stream(inbound)
            -> deserialize_input
            -> demux(|m, var_args!(puts, gets)| match m {
                KVSRequest::Put {key, value} => puts.give((key, value)),
                KVSRequest::Get {key} => gets.give((key, ())),
            });

        // join PUTs and GETs by key
        lookup = join::<'static, 'tick>();
        my_demux[puts] -> [0]lookup;
        my_demux[gets] -> [1]lookup;

        // send lookup responses back to the client address from the GET
        lookup -> map(|(key, (value, ()))| KVSResponse::Response{key, value}) -> serialize_output -> dest_sink(outbound);
    };

    df.run_async().await;
}
