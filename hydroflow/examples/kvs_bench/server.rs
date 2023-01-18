use super::KVSRequest;
use super::KVSResponse;
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
        outbound_chan = dest_sink_serde_tcp(outbound);
        my_demux = source_stream_serde_tcp(inbound)
            -> demux(|m, var_args!(puts, gets)| match m {
                KVSRequest::Put {..} => puts.give(m),
                KVSRequest::Get {..} => gets.give(m),
            });

        parsed_puts = my_demux[puts] -> filter_map(|m| {
            match m {
                KVSRequest::Put{key, value} => Some((key, value)),
                _ => None
            }
        });

        parsed_gets = my_demux[gets] -> filter_map(|m| {
            match m {
                KVSRequest::Get{key} => Some((key, ())),
                _ => None
            }
        });

        // join PUTs and GETs by key
        lookup = join::<'static, 'tick>();
        parsed_puts -> [0]lookup;
        parsed_gets -> [1]lookup;

        // send lookup responses back to the client address from the GET
        lookup -> map(|(key, (value, ()))| KVSResponse::Response{key, value}) -> outbound_chan;
    };

    df.run_async().await;
}
