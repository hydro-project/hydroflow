use chrono::prelude::*;
use hydroflow::futures::Stream;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::tokio::sync::mpsc::UnboundedSender;

use crate::protocol::Msg;

fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 2) + fibonacci(n - 1)
    }
}

#[tokio::main]
pub(crate) async fn run_server(
    inbound: impl 'static + Stream<Item = Msg>,
    outbound: UnboundedSender<Msg>,
) {
    println!("Server live!");

    let mut flow: Hydroflow = hydroflow_syntax! {
        // Define a shared inbound channel
        inbound_chan = source_stream(inbound);

        // Echo back the Echo messages with updated timestamp
        inbound_chan
            // Print all messages for debugging purposes
            -> inspect(|msg| println!("Server {}: Got {}=>{}", Utc::now(), msg.idx, msg.val))
            -> map(|msg| Msg { val: fibonacci(msg.val), ..msg }) -> for_each(|msg| outbound.send(msg).unwrap());
    };

    // run the server
    flow.run_async().await;
}
