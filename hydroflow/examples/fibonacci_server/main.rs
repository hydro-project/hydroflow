use std::net::{TcpListener, TcpStream};

use client::run_client;
use futures::StreamExt;
use server::run_server;
use tokio::runtime::Builder;

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

    // let (req_send, req_recv) = hydroflow::util::unbounded_channel();
    // let (resp_send, resp_recv) = hydroflow::util::unbounded_channel();
    let tcp_serv = TcpListener::bind("127.0.0.1:0").unwrap();
    let serv_addr = tcp_serv.local_addr().unwrap();

    let server = std::thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        rt.block_on(async {
            let (tcp_accept, _addr) = tcp_serv.accept().unwrap();
            tcp_accept.set_nonblocking(true).unwrap();
            let (resp_send, req_recv) =
                hydroflow::util::tcp_bytes(tokio::net::TcpStream::from_std(tcp_accept).unwrap());
            let req_recv = req_recv.map(Result::unwrap);
            run_server(req_recv, resp_send).await
        })
    });
    let client_a_handle = std::thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        rt.block_on(async {
            let tcp_connect = TcpStream::connect(serv_addr).unwrap();
            tcp_connect.set_nonblocking(true).unwrap();
            let (req_send, resp_recv) =
                hydroflow::util::tcp_bytes(tokio::net::TcpStream::from_std(tcp_connect).unwrap());
            let resp_recv = resp_recv.map(Result::unwrap);
            run_client(resp_recv, req_send).await
        })
    });

    server.join().unwrap();
    client_a_handle.join().unwrap();
}
