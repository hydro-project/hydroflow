use hydroflow::tokio::sync::mpsc::UnboundedSender;
use hydroflow::tokio_stream::wrappers::UnboundedReceiverStream;
use hydroflow_plus::deploy::SingleProcessGraph;
use hydroflow_plus::*;
use stream::NoOrder;

#[stageleft::entry]
pub fn graph_reachability<'a>(
    flow: FlowBuilder<'a>,
    roots: RuntimeData<UnboundedReceiverStream<u32>>,
    edges: RuntimeData<UnboundedReceiverStream<(u32, u32)>>,
    reached_out: RuntimeData<&'a UnboundedSender<u32>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let process = flow.process::<()>();

    let roots = process.source_stream(roots);
    let edges = process.source_stream(edges);

    let reachability_tick = process.tick();
    let (set_reached_cycle, reached_cycle) = reachability_tick.cycle::<Stream<_, _, _, NoOrder>>();

    let reached = unsafe {
        // SAFETY: TODO
        roots.tick_batch(&reachability_tick).union(reached_cycle)
    };
    let reachable = reached
        .clone()
        .map(q!(|r| (r, ())))
        .join(unsafe {
            // SAFETY: TODO
            edges.tick_batch(&reachability_tick).persist()
        })
        .map(q!(|(_from, (_, to))| to));
    set_reached_cycle.complete_next_tick(reached.clone().union(reachable));

    reached.all_ticks().unique().for_each(q!(|v| {
        reached_out.send(v).unwrap();
    }));

    flow.compile_no_network::<SingleProcessGraph>()
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydroflow::assert_graphvis_snapshots;
    use hydroflow::util::collect_ready;

    #[test]
    pub fn test_reachability() {
        let (roots_send, roots) = hydroflow::util::unbounded_channel();
        let (edges_send, edges) = hydroflow::util::unbounded_channel();
        let (out, mut out_recv) = hydroflow::util::unbounded_channel();

        let mut reachability = super::graph_reachability!(roots, edges, &out);
        assert_graphvis_snapshots!(reachability);

        roots_send.send(1).unwrap();
        roots_send.send(2).unwrap();

        edges_send.send((1, 2)).unwrap();
        edges_send.send((2, 3)).unwrap();
        edges_send.send((3, 4)).unwrap();
        edges_send.send((4, 5)).unwrap();

        reachability.run_tick();
        reachability.run_tick();
        reachability.run_tick();
        reachability.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[1, 2, 3, 4, 5]
        );
    }
}
