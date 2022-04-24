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
    let mut data = [0 as u8; 1000];
    loop {
        match stream.read(&mut data) {
            Ok(_) => {
                //println!("received");
            }
            Err(e) => {
                println!("Unable to process request: {}", e);
                stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }
}
