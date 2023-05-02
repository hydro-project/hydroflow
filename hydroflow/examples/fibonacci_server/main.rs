use client::run_client;
use server::run_server;

mod client;
mod protocol;
mod server;

fn main() {
    // // parse command line arguments
    // let opts = Opts::parse();
    // // if no addr was provided, we ask the OS to assign a local port by passing in "localhost:0"
    // let addr = opts
    //     .addr
    //     .unwrap_or_else(|| ipv4_resolve("localhost:0").unwrap());

    // // allocate `outbound` sink and `inbound` stream
    // let (outbound, inbound, addr) = bind_udp_bytes(addr).await;
    // println!("Listening on {:?}", addr);

    let (req_send, req_recv) = hydroflow::util::unbounded_channel();
    let (resp_send, resp_recv) = hydroflow::util::unbounded_channel();

    let server = std::thread::spawn(|| run_server(req_recv, resp_send));
    let client_a_handle = std::thread::spawn(|| run_client(resp_recv, req_send));

    server.join().unwrap();
    client_a_handle.join().unwrap();
}
