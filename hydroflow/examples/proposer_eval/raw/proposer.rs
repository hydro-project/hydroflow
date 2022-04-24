use crate::protocol::{ClientReq, Msg, MsgType, ProposerReq};
use crate::Opts;
use std::io::{Read, Write};
use std::net::TcpStream;

pub async fn run(opts: Opts) {
    match TcpStream::connect(opts.addr.clone()) {
        Ok(mut stream) => {
            let mut slot = 0;
            let msg = Msg::ProposerReq(ProposerReq {
                addr: opts.addr,
                slot,
                ballot: 0,
                pid: 0,
                val: 0,
                mtype: MsgType::P1A,
            });

            let msg = bincode::serialize(&msg).unwrap();
            match stream.write(msg.as_slice()) {
                Ok(_) => (),
                Err(e) => println!("Unable to send a request: {}", e),
            }
            slot += 1;
        }
        Err(e) => {
            println!("Unable to start the proposer: {}", e)
        }
    }
}
