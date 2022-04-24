use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::{Duration, SystemTime};

pub fn run(addr: String, port: u16) {
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_conn(stream, port);
    }
}

fn handle_conn(mut stream: TcpStream, port: u16) {
    let mut start = SystemTime::now();
    let mut counter = 0;
    let mut total_counter = 0;
    let mut warmup = true;

    loop {
        // TODO: KEEP THIS IN SYNC with the proposer.rs msg.
        // TODO: refactor
        let mut buf = [0 as u8; 75];

        stream.read(&mut buf).unwrap();
        let received = String::from_utf8_lossy(&buf);

        counter += 1;
        total_counter += 1;
        if counter % 10000 == 0 {
            if warmup {
                start = SystemTime::now();
                counter = 0;
                warmup = false;
                continue;
            }

            let elapsed = start.elapsed().unwrap();
            let elapsed_ms =
                elapsed.as_secs() as f64 * 1000.0 + elapsed.subsec_nanos() as f64 / 1_000_000.0;

            println!(
                "Acceptor on port {}, Counter {}, Elapsed {}, Throughput {}",
                port,
                total_counter,
                elapsed_ms,
                counter as f64 / elapsed_ms as f64 * 1000.0
            );
        }

        //if i % 10000 == 0 {
        //println!("Acceptor received msgs {}", i);
        //}
    }
}
