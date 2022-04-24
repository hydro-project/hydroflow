use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};

pub fn run(addr: String) {
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_conn(stream);
    }
}

fn handle_conn(mut stream: TcpStream) {
    let mut i = 0;
    loop {
        let mut buf = [0 as u8; 75];
        // TODO: KEEP THIS IN SYNC with the proposer.rs msg.
        // TODO: refactor
        stream.read(&mut buf).unwrap();
        let received = String::from_utf8_lossy(&buf);
        i += 1;

        //if i % 10000 == 0 {
        println!("Acceptor received msgs {}", i);
        //}
    }
}
