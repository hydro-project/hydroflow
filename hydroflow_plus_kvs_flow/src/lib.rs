use hydroflow_plus::HfBuilder;
use stagefright::*;

quse!(::hydroflow_plus::hydroflow::tokio_stream::wrappers::UnboundedReceiverStream);
quse!(::hydroflow_plus::hydroflow::scheduled::graph::Hydroflow);
quse!(::hydroflow_plus::hydroflow::bytes::Bytes);
quse!(::hydroflow_plus::hydroflow::util);
quse!(::hydroflow_plus_kvs_types::KVSMessage);

#[stagefright::entry]
pub fn my_example_flow<'a>(
    graph: &'a HfBuilder<'a>,
    input_stream: RuntimeData<UnboundedReceiverStream<Bytes>>,
) -> impl Quoted<Hydroflow<'a>> {
    let inbound_channel =
        graph
            .source_stream(q!(input_stream))
            .map(q!(|bytes| util::deserialize_from_bytes::<KVSMessage>(
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

    puts.for_each(q!(|msg| {
        println!("Got a Put {:?}", msg);
    }));

    gets.for_each(q!(|msg| {
        println!("Got a Get {:?}", msg);
    }));

    graph.build()
}
