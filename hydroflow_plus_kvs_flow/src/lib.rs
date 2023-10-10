#![cfg_attr(feature = "final", allow(unused))]

#[cfg(feature = "final")]
#[doc(hidden)]
pub(crate) use hydroflow_plus_kvs_macro as __macro;

#[cfg(feature = "final")]
#[doc(hidden)]
pub mod __flow {
    include!(concat!(env!("OUT_DIR"), "/lib_pub.rs"));
}

use hydroflow_plus::hydroflow::bytes::Bytes;
use hydroflow_plus::hydroflow::scheduled::graph::Hydroflow;
use hydroflow_plus::hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow_plus::hydroflow::util;
use hydroflow_plus::HfBuilder;
use serde::{Deserialize, Serialize};
use stageleft::{q, Quoted, RuntimeData};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSMessage {
    Put { key: String, value: String },
    Get { key: String },
    Response { key: String, value: String },
}

#[stageleft::entry]
pub fn my_kvs<'a>(
    graph: &'a HfBuilder<'a>,
    enable_debug: bool,
    input_stream: RuntimeData<UnboundedReceiverStream<Bytes>>,
) -> impl Quoted<Hydroflow<'a>> {
    let input_bytes = graph.source_stream(q!(input_stream));

    let inbound_channel =
        input_bytes.map(q!(|bytes| util::deserialize_from_bytes::<KVSMessage>(
            bytes
        )
        .unwrap()));

    let gets = inbound_channel.filter_map(q!(|msg| match msg {
        KVSMessage::Get { key } => Some(key),
        _ => None,
    }));

    let puts = inbound_channel.filter_map(q!(|msg| match msg {
        KVSMessage::Put { key, value } => Some((key, value)),
        _ => None,
    }));

    if enable_debug {
        puts.for_each(q!(|msg| {
            println!("Got a Put {:?}", msg);
        }));

        gets.for_each(q!(|msg| {
            println!("Got a Get {:?}", msg);
        }));
    }

    graph.build()
}
