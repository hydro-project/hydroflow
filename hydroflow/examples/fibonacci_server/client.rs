use crate::protocol::Msg;

use std::fmt::Debug;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use chrono::prelude::*;
use hydroflow::futures::{Sink, Stream, StreamExt};
use hydroflow::hydroflow_syntax;
use hydroflow::tokio_stream::wrappers::IntervalStream;
use rand::Rng;

pub(crate) async fn run_client<In, Out>(inbound: In, outbound: Out)
where
    In: 'static + Stream<Item = BytesMut>,
    Out: 'static + Sink<Bytes> + Send + Unpin,
    Out::Error: Debug,
{
    // // server_addr is required for client
    // let server_addr = opts.server_addr.expect("Client requires a server address");
    println!("Client live!");

    let input = IntervalStream::new(tokio::time::interval(Duration::from_millis(10)))
        .take(1_000)
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
        inbound_chan = source_stream(inbound)
            -> map(hydroflow::util::deserialize_from_bytes)
            -> map(Result::unwrap);
        outbound_chan = // for_each(|msg| outbound.send(msg).unwrap());
            map(hydroflow::util::serialize_to_bytes)
            -> dest_sink(outbound);

        // Print all messages for debugging purposes
        inbound_chan
            -> for_each(|msg: Msg| println!("Client {}: Got {}=>{}", Utc::now(), msg.idx, msg.val));

        // // take stdin and send to server as an Message::Echo
        // source_stdin() -> map(|l| (l.unwrap().parse::<u64>().unwrap(), server_addr) )
        source_stream(input) // -> map(|n| (n, server_addr))
            -> inspect(|msg| println!("Client {}: Send {}=>{}", Utc::now(), msg.idx, msg.val))
            -> outbound_chan;
    };

    flow.run_async().await.unwrap();
}
