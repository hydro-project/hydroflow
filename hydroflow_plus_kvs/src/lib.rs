#[cfg(not(feature = "macro"))]
stageleft::stageleft_crate!(hydroflow_plus_kvs_macro);

use hydroflow_plus::hydroflow::bytes::Bytes;
use hydroflow_plus::hydroflow::scheduled::graph::Hydroflow;
use hydroflow_plus::hydroflow::tokio::sync::mpsc::UnboundedSender;
use hydroflow_plus::hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow_plus::hydroflow::util;
use hydroflow_plus::{HfBuilder, HfStream};
use regex::Regex;
use serde::{Deserialize, Serialize};
use stageleft::{q, IntoQuotedOnce, Quoted, RuntimeData};

struct Test {
    pub v: String,
}

fn filter_by_regex<'a, S: Copy + AsRef<str> + 'a>(
    graph: &HfBuilder<'a>,
    input: HfStream<'a, String>,
    pattern: RuntimeData<S>,
) -> HfStream<'a, String> {
    let ctx = graph.runtime_context();

    input.filter(q!({
        let regex = Regex::new(pattern.as_ref()).unwrap();
        move |x| {
            dbg!(ctx.current_tick());
            let constructed_test = Test { v: x.clone() };
            dbg!(constructed_test.v);
            regex.is_match(x)
        }
    }))
}

#[stageleft::entry(&'static str)]
pub fn my_example_flow<'a, S: Copy + AsRef<str> + 'a>(
    graph: &'a HfBuilder<'a>,
    input_stream: RuntimeData<UnboundedReceiverStream<String>>,
    output_sink: RuntimeData<&'a UnboundedSender<String>>,
    number_of_foreach: u32,
    regex: RuntimeData<S>,
    text: RuntimeData<&'a str>,
) -> impl Quoted<Hydroflow<'a>> {
    let ctx = graph.runtime_context();
    let source = graph.source_stream(q!(input_stream));

    let mapped = filter_by_regex(graph, source, regex);

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("passed regex {} {}", text, x)));
    }

    mapped.for_each(q!(|x| {
        output_sink.send(x).unwrap();
    }));

    graph.build()
}

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

    let inbound_channel = input_bytes.map(q!(|bytes| util::deserialize_from_bytes::<KVSMessage>(
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

#[stageleft::entry]
pub fn raise_to_power(ctx: &(), value: RuntimeData<i32>, power: u32) -> impl Quoted<i32> {
    if power == 1 {
        q!(value).boxed()
    } else if power % 2 == 0 {
        let half_result = raise_to_power(ctx, value, power / 2);
        q!({
            let v = half_result;
            v * v
        })
        .boxed()
    } else {
        let half_result = raise_to_power(ctx, value, power / 2);
        q!({
            let v = half_result;
            (v * v) * value
        })
        .boxed()
    }
}
