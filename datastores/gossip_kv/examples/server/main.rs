use hydroflow::{hydroflow_syntax, tokio};
use hydroflow::util::{UdpSink, UdpStream};
use hydroflow::util::{bind_tcp_lines, ipv4_resolve};

#[hydroflow::main]
async fn main() {
    // Start admin port

    let my_hostname = hostname::get().unwrap();

    let (tx, rx, server_addr) =
        bind_tcp_lines("0.0.0.0:80".parse().unwrap()).await;

    println!("Starting admin server");

    let mut echo_server = hydroflow_syntax! {
        source_stream(rx)
            -> filter_map(Result::ok)
            -> inspect(|(msg, addr)| { println!("Received {:?} from {:?}", msg, addr)} )
            -> map(|(msg, addr)| { (format!("Echo from: {:?}: {:?}", my_hostname, msg), addr) })
            -> dest_sink(tx);
    };

    println!("Admin server running on {:?}", server_addr);

    echo_server.run_async().await;
}
