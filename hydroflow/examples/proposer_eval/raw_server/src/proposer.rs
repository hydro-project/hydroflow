use crate::protocol::{Msg, MsgType, ProposerReq};
use crate::Opts;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime};

fn hash_u16(x: u16) -> u64 {
    let mut s = DefaultHasher::new();
    x.hash(&mut s);
    s.finish()
}

fn calculate_hash(val: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

fn waste_time(val: u64) -> u64 {
    let mut current_val = calculate_hash(val);
    for _ in 0..1_000 {
        let new_val = calculate_hash(current_val);
        current_val = new_val;
    }
    current_val
}

pub fn run(opts: Opts) {
    let mut streams = Vec::new();
    if opts.use_proxy {
        for addr in opts.proxy_addrs.iter() {
            match TcpStream::connect(addr.clone()) {
                Ok(stream) => {
                    streams.push(stream);
                }
                Err(e) => {
                    println!("Unable to start the proposer: {}", e)
                }
            }
        }
    } else {
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
    }

    let mut total_counter = 0;
    let mut counter = 0;
    let mut rng = rand::thread_rng();
    let mut start = SystemTime::now();
    let mut warmup = true;

    let msg =
        String::from("I need some fixed length of bytes that is comparable to a proposer message.");
    while total_counter < 300000 + 100 {
        waste_time(hash_u16((total_counter) as u16));

        if opts.use_proxy {
            let stream_index = total_counter % streams.len();
            match streams[stream_index].write(msg.as_bytes()) {
                Ok(_) => (),
                Err(e) => println!("Unable to send a request: {}", e),
            }
        } else {
            for (i, mut stream) in streams.iter().enumerate() {
                //let msg = Msg::ProposerReq(ProposerReq {
                //addr: opts.acceptor_addrs[i].clone(),
                //slot: total_counter as i32,
                //ballot: 0,
                //pid: 0,
                //val: rng.gen(),
                //mtype: MsgType::P1A,
                //});

                //let msg = bincode::serialize(&msg).unwrap();
                match stream.write(msg.as_bytes()) {
                    Ok(_) => (),
                    Err(e) => println!("Unable to send a request: {}", e),
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
        counter += 1;
        total_counter += 1;
        if counter % 10000 == 0 {
            let elapsed = start.elapsed().unwrap();
            let elapsed_ms =
                elapsed.as_secs() as f64 * 1000.0 + elapsed.subsec_nanos() as f64 / 1_000_000.0;
            println!(
                "Counter {}, Elapsed {}, Throughput {}",
                total_counter,
                elapsed_ms,
                counter as f64 / elapsed_ms as f64 * 1000.0
            );
            if warmup {
                start = SystemTime::now();
                counter = 0;
                warmup = false;
            }
        }
    }
}
