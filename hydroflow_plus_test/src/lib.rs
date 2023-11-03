stageleft::stageleft_crate!(hydroflow_plus_test_macro);

use hydroflow_plus::futures::stream::Stream;
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::tokio::sync::mpsc::UnboundedSender;
use hydroflow_plus::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow_plus::HfBuilder;
use stageleft::{q, Quoted, RuntimeData};

#[stageleft::entry(UnboundedReceiverStream<u32>)]
pub fn teed_join<'a, S: Stream<Item = u32> + Unpin + 'a>(
    graph: &'a HfBuilder<'a>,
    input_stream: RuntimeData<S>,
    output: RuntimeData<&'a UnboundedSender<u32>>,
) -> impl Quoted<Hydroflow<'a>> {
    let source = graph.source_stream(q!(input_stream));
    let map1 = source.map(q!(|v| (v + 1, ())));
    let map2 = source.map(q!(|v| (v - 1, ())));

    let joined = map1.join(&map2).map(q!(|t| t.0));

    joined.for_each(q!(|v| {
        output.send(v).unwrap();
    }));

    graph.build()
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydroflow_plus::util::collect_ready;

    use super::*;

    #[test]
    fn test_teed_join() {
        let (in_send, input) = hydroflow_plus::util::unbounded_channel::<u32>();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel::<u32>();

        let mut joined = teed_join!(input, &out);

        in_send.send(1).unwrap();
        in_send.send(2).unwrap();
        in_send.send(3).unwrap();
        in_send.send(4).unwrap();

        joined.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[2, 3]);
    }
}
