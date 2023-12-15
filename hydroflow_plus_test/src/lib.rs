stageleft::stageleft_crate!(hydroflow_plus_test_macro);

use hydroflow_plus::futures::stream::Stream;
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::tokio::sync::mpsc::UnboundedSender;
use hydroflow_plus::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow_plus::HfBuilder;
use stageleft::{q, Quoted, RuntimeData};

pub mod networked;

#[stageleft::entry(UnboundedReceiverStream<u32>)]
pub fn teed_join<'a, S: Stream<Item = u32> + Unpin + 'a>(
    graph: &'a HfBuilder<'a>,
    input_stream: RuntimeData<S>,
    output: RuntimeData<&'a UnboundedSender<u32>>,
    send_twice: bool,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let source = graph.source_stream(&0, input_stream);
    let map1 = source.map(q!(|v| (v + 1, ())));
    let map2 = source.map(q!(|v| (v - 1, ())));

    let joined = map1.join(&map2).map(q!(|t| t.0));

    joined.for_each(q!(|v| {
        output.send(v).unwrap();
    }));

    if send_twice {
        joined.for_each(q!(|v| {
            output.send(v).unwrap();
        }));
    }

    let source_node_id_1 = graph.source_iter(&1, q!(0..5));
    source_node_id_1.for_each(q!(|v| {
        output.send(v).unwrap();
    }));

    graph.build(node_id)
}

#[stageleft::entry]
pub fn chat_app<'a>(
    graph: &'a HfBuilder<'a>,
    users_stream: RuntimeData<UnboundedReceiverStream<u32>>,
    messages: RuntimeData<UnboundedReceiverStream<String>>,
    output: RuntimeData<&'a UnboundedSender<(u32, String)>>,
    replay_messages: bool,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let users = graph.source_stream(&(), users_stream).persist();
    let mut messages = graph.source_stream(&(), messages);
    if replay_messages {
        messages = messages.persist();
    }

    let mut joined = users.cross_product(&messages);
    if replay_messages {
        joined = joined.delta();
    }

    joined.for_each(q!(|t| {
        output.send(t).unwrap();
    }));

    graph.build(q!(0))
}

#[stageleft::entry]
pub fn graph_reachability<'a>(
    graph: &'a HfBuilder<'a>,
    roots: RuntimeData<UnboundedReceiverStream<u32>>,
    edges: RuntimeData<UnboundedReceiverStream<(u32, u32)>>,
    reached_out: RuntimeData<&'a UnboundedSender<u32>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let roots = graph.source_stream(&(), roots);
    let edges = graph.source_stream(&(), edges);

    let (set_reached_cycle, reached_cycle) = graph.cycle(&());

    let reached = roots.union(&reached_cycle);
    let reachable = reached
        .map(q!(|r| (r, ())))
        .join(&edges)
        .map(q!(|(_from, (_, to))| to));
    set_reached_cycle.complete(&reachable);

    reached.unique().for_each(q!(|v| {
        reached_out.send(v).unwrap();
    }));

    graph.build(q!(0))
}

#[stageleft::entry(String)]
pub fn count_elems<'a, T: 'a>(
    graph: &'a HfBuilder<'a>,
    input_stream: RuntimeData<UnboundedReceiverStream<T>>,
    output: RuntimeData<&'a UnboundedSender<u32>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let source = graph.source_stream(&(), input_stream);
    let count = source.map(q!(|_| 1)).fold(q!(|| 0), q!(|a, b| *a += b));

    count.for_each(q!(|v| {
        output.send(v).unwrap();
    }));

    graph.build(q!(0))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydroflow_plus::assert_graphvis_snapshots;
    use hydroflow_plus::util::collect_ready;

    use super::*;

    #[test]
    fn test_teed_join() {
        let (in_send, input) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut joined = teed_join!(input, &out, false, 0);
        assert_graphvis_snapshots!(joined);

        in_send.send(1).unwrap();
        in_send.send(2).unwrap();
        in_send.send(3).unwrap();
        in_send.send(4).unwrap();

        joined.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[2, 3]);
    }

    #[test]
    fn test_teed_join_twice() {
        let (in_send, input) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut joined = teed_join!(input, &out, true, 0);
        assert_graphvis_snapshots!(joined);

        in_send.send(1).unwrap();
        in_send.send(2).unwrap();
        in_send.send(3).unwrap();
        in_send.send(4).unwrap();

        joined.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[2, 2, 3, 3]);
    }

    #[test]
    fn test_teed_join_multi_node() {
        let (_, input) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut joined = teed_join!(input, &out, true, 1);
        assert_graphvis_snapshots!(joined);

        joined.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn test_chat_app_no_replay() {
        let (users_send, users) = hydroflow_plus::util::unbounded_channel();
        let (messages_send, messages) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut chat_server = chat_app!(users, messages, &out, false);
        assert_graphvis_snapshots!(chat_server);

        users_send.send(1).unwrap();
        users_send.send(2).unwrap();

        messages_send.send("hello".to_string()).unwrap();
        messages_send.send("world".to_string()).unwrap();

        chat_server.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[
                (1, "hello".to_string()),
                (2, "hello".to_string()),
                (1, "world".to_string()),
                (2, "world".to_string())
            ]
        );

        users_send.send(3).unwrap();

        messages_send.send("goodbye".to_string()).unwrap();

        chat_server.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[
                (1, "goodbye".to_string()),
                (2, "goodbye".to_string()),
                (3, "goodbye".to_string())
            ]
        );
    }

    #[test]
    fn test_chat_app_replay() {
        let (users_send, users) = hydroflow_plus::util::unbounded_channel();
        let (messages_send, messages) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut chat_server = chat_app!(users, messages, &out, true);
        assert_graphvis_snapshots!(chat_server);

        users_send.send(1).unwrap();
        users_send.send(2).unwrap();

        messages_send.send("hello".to_string()).unwrap();
        messages_send.send("world".to_string()).unwrap();

        chat_server.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[
                (1, "hello".to_string()),
                (2, "hello".to_string()),
                (1, "world".to_string()),
                (2, "world".to_string())
            ]
        );

        users_send.send(3).unwrap();

        messages_send.send("goodbye".to_string()).unwrap();

        chat_server.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[
                (3, "hello".to_string()),
                (3, "world".to_string()),
                (1, "goodbye".to_string()),
                (2, "goodbye".to_string()),
                (3, "goodbye".to_string())
            ]
        );
    }

    #[test]
    pub fn test_reachability() {
        let (roots_send, roots) = hydroflow_plus::util::unbounded_channel();
        let (edges_send, edges) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut reachability = graph_reachability!(roots, edges, &out);
        assert_graphvis_snapshots!(reachability);

        roots_send.send(1).unwrap();
        roots_send.send(2).unwrap();

        edges_send.send((1, 2)).unwrap();
        edges_send.send((2, 3)).unwrap();
        edges_send.send((3, 4)).unwrap();
        edges_send.send((4, 5)).unwrap();

        reachability.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[1, 2, 3, 4, 5]
        );
    }

    #[test]
    pub fn test_count() {
        let (in_send, input) = hydroflow_plus::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut count = count_elems!(input, &out);
        assert_graphvis_snapshots!(count);

        in_send.send(1).unwrap();
        in_send.send(1).unwrap();
        in_send.send(1).unwrap();

        count.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[3]);
    }
}
