use crate::Opts;
use std::net::TcpListener;
use std::net::TcpStream;

pub async fn run(opts: Opts) {
    let listener = TcpListener::bind(opts.addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_conn(stream);
    }
}

fn handle_conn(mut stream: TcpStream) {
    // Count
}
