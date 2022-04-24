use crate::Opts;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};

pub fn run(addr: String, opts: Opts) {
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_conn(stream, opts.clone());
    }
}

fn handle_conn(mut stream: TcpStream, opts: Opts) {
    let mut streams = Vec::new();
    for addr in opts.acceptor_addrs.iter() {
        match TcpStream::connect(addr.clone()) {
            Ok(stream) => {
                streams.push(stream);
            }
            Err(e) => {
                println!("Unable to start the proposer: {}", e)
            }
        }
    }

    loop {
        // TODO: KEEP THIS IN SYNC with the proposer.rs msg.
        // TODO: refactor
        let mut buf = [0 as u8; 75];
        stream.read(&mut buf).unwrap();
        let received = String::from_utf8_lossy(&buf);

        for (i, mut stream) in streams.iter().enumerate() {
            match stream.write(received.as_bytes()) {
                Ok(_) => (),
                Err(e) => println!("Unable to send a request: {}", e),
            }
        }
    }
}
