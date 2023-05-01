use crate::protocol::Msg;
use crate::Opts;
use chrono::prelude::*;
use hydroflow::futures::StreamExt;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{UdpSink, UdpStream};
use rand::Rng;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;

pub(crate) async fn run_client(outbound: UdpSink, inbound: UdpStream, opts: Opts) {
    // server_addr is required for client
    let server_addr = opts.server_addr.expect("Client requires a server address");
    println!("Client live!");

    let input = IntervalStream::new(tokio::time::interval(Duration::from_millis(10)))
        .map(|_| rand::thread_rng().gen_range(0..=40) as u64)
        .enumerate()
        .map(|(idx, val)| Msg {
            val,
            idx: idx as u64,
            ts: Utc::now(),
            nonce: vec![0; 16_000],
        });

    let mut flow = hydroflow_syntax! {
        // Define shared inbound and outbound channels
        inbound_chan = source_stream_serde(inbound);
        outbound_chan = dest_sink_serde(outbound);

        // Print all messages for debugging purposes
        inbound_chan
            -> map(Result::unwrap)
            -> for_each(|(msg, addr): (Msg, SocketAddr)| println!("{}: Got {}=>{} from {:?}", Utc::now(), msg.idx, msg.val, addr));

        // // take stdin and send to server as an Message::Echo
        // source_stdin() -> map(|l| (l.unwrap().parse::<u64>().unwrap(), server_addr) )
        source_stream(input) -> map(|n| (n, server_addr))
            -> inspect(|(msg, addr)| println!("{}: Send {}=>{} from {:?}", Utc::now(), msg.idx, msg.val, addr))
            -> outbound_chan;
    };

    flow.run_async().await.unwrap();
}
