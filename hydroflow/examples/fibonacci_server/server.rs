use bytes::{Bytes, BytesMut};
use chrono::prelude::*;
use hydroflow::futures::{Sink, Stream};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use std::fmt::Debug;

use crate::protocol::Msg;

fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 2) + fibonacci(n - 1)
    }
}

pub(crate) async fn run_server<In, Out>(inbound: In, outbound: Out)
where
    In: 'static + Stream<Item = BytesMut>,
    Out: 'static + Sink<Bytes> + Send + Unpin,
    Out::Error: Debug,
{
    println!("Server live!");

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Define a shared inbound channel
        inbound_chan = source_stream(inbound)
            -> map(hydroflow::util::deserialize_from_bytes)
            -> map(Result::unwrap);

        // Echo back the Echo messages with updated timestamp
        inbound_chan
            // Print all messages for debugging purposes
            -> inspect(|msg: &Msg| println!("Server {}: Got {}=>{}", Utc::now(), msg.idx, msg.val))
            -> map(|msg| Msg { val: fibonacci(msg.val), ..msg })
            -> map(hydroflow::util::serialize_to_bytes)
            -> dest_sink(outbound);
            // -> for_each(|msg| outbound.send(msg).unwrap());
    };

    // run the server
    flow.run_async().await;
}
