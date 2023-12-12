use hydroflow::bytes::BytesMut;
use hydroflow::util::cli::{ConnectedDirect, ConnectedSink, ConnectedSource, HydroCLI};
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::HfBuilder;
use stageleft::{q, Quoted, RuntimeData};

#[stageleft::entry]
pub fn networked_basic<'a>(
    graph: &'a HfBuilder<'a>,
    cli: RuntimeData<&'a HydroCLI>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let source_zero = graph.source_stream(
        0,
        q!({
            cli.port("node_zero_input")
                .connect_local_blocking::<ConnectedDirect>()
                .into_source()
        }),
    );

    source_zero
        .map(q!(|v: Result<BytesMut, _>| v.unwrap().freeze()))
        .dest_sink(q!({
            cli.port("node_zero_output")
                .connect_local_blocking::<ConnectedDirect>()
                .into_sink()
        }));

    let source_one = graph.source_stream(
        1,
        q!({
            cli.port("node_one_input")
                .connect_local_blocking::<ConnectedDirect>()
                .into_source()
        }),
    );

    source_one.for_each(q!(|v: Result<BytesMut, _>| {
        println!(
            "node one received: {:?}",
            std::str::from_utf8(&v.unwrap()).unwrap()
        );
    }));

    graph.build(node_id)
}
