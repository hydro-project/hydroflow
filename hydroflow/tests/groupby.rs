use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::handoff::VecHandoff;

/// First batch of items for monotonic threshold test.
const BATCH_A: [&'static str; 7] = ["megan", "davis", "mingwei", "john", "justin", "joe", "mae"];
/// Second batch.
const BATCH_B: [&'static str; 7] = [
    "mingwei", "lauren", "justin", "mae", "mingwei", "justin", "pierce",
];
/// Third & final batch.
const BATCH_C: [&'static str; 3] = ["joe", "mae", "zach"];

/// Basic monotonic threshold: release a value once after it has been seen three times.
/// Uses the core API.
#[test]
fn groupby_monotonic_core() {
    let mut hf = Hydroflow::new();

    let (source_send, source_recv) = hf.make_edge::<_, VecHandoff<&'static str>>("source handoff");
    let input = hf.add_input("source", source_send);

    let (sink_send, sink_recv) = hf.make_edge::<_, VecHandoff<&'static str>>("sink handoff");

    let mut groups = HashMap::<&'static str, u32>::new();
    hf.add_subgraph_in_out(
        "group by",
        source_recv,
        sink_send,
        move |_ctx, recv, send| {
            for item in recv.take_inner() {
                let count = groups.entry(item).or_default();
                *count += 1;
                if 3 == *count {
                    send.give(Some(item));
                }
            }
        },
    );

    let output = <Rc<RefCell<Vec<&'static str>>>>::default();
    let output_ref = output.clone();
    hf.add_subgraph_sink("sink", sink_recv, move |_ctx, recv| {
        for v in recv.take_inner().into_iter() {
            output_ref.borrow_mut().push(v);
        }
    });

    input.give(Iter(BATCH_A.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(0, output.borrow().len());

    input.give(Iter(BATCH_B.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["mingwei", "justin"], &**output.borrow());

    input.give(Iter(BATCH_C.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["mingwei", "justin", "mae"], &**output.borrow());
}

/// Basic monotonic threshold: release a value once after it has been seen three times.
/// Uses the surface (builder) API.
#[test]
fn groupby_monotonic_surface() {
    use hydroflow::builder::prelude::*;

    let mut hf_builder = HydroflowBuilder::new();
    let (input, source_recv) =
        hf_builder.add_channel_input::<_, _, VecHandoff<&'static str>>("source");

    let output = <Rc<RefCell<Vec<&'static str>>>>::default();
    let output_ref = output.clone();

    hf_builder.add_subgraph(
        "main",
        source_recv
            .flatten()
            .map_scan(HashMap::new(), |groups, item| {
                let count = groups.entry(item).or_default();
                *count += 1;
                (item, *count)
            })
            .filter_map(|(item, count)| if 3 == count { Some(item) } else { None })
            .pull_to_push()
            .for_each(move |item| output_ref.borrow_mut().push(item)),
    );

    let mut hf = hf_builder.build();

    input.give(Iter(BATCH_A.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(0, output.borrow().len());

    input.give(Iter(BATCH_B.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["mingwei", "justin"], &**output.borrow());

    input.give(Iter(BATCH_C.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["mingwei", "justin", "mae"], &**output.borrow());
}

/// Non-monotonic barrier. Find the median name.
/// Takes in BATCH_A in the first epoch, then BATCH_B *and* BATCH_C in the second epoch.
#[test]
fn groupby_nonmon_surface() {
    use hydroflow::builder::prelude::*;

    let mut hf_builder = HydroflowBuilder::new();
    let (input, source_recv) =
        hf_builder.add_channel_input::<_, _, VecHandoff<&'static str>>("source");

    let (a_send, a_recv) = hf_builder.make_edge::<_, VecHandoff<&'static str>, _>("names a-m");
    let (z_send, z_recv) = hf_builder.make_edge::<_, VecHandoff<&'static str>, _>("names n-z");
    let (stratum_boundary_send, stratum_boundary_recv) =
        hf_builder.make_edge::<_, VecHandoff<&'static str>, _>("names n-z");

    // Make output first, to mess with scheduler order.
    let output = <Rc<Cell<Option<(usize, &'static str)>>>>::default();
    let output_ref = output.clone();
    hf_builder.add_subgraph_stratified(
        "find median",
        1,
        stratum_boundary_recv
            .map(|mut buffer| {
                let batch_size = buffer.len();
                let median = *buffer
                    .make_contiguous()
                    .select_nth_unstable(batch_size / 2)
                    .1;
                (batch_size, median)
            })
            .pull_to_push()
            .map(Some)
            .for_each(move |val| output_ref.set(val)),
    );

    // Partition then re-merge names to make graph more interesting.
    // Want to have multiple compiled components to test scheduler.
    hf_builder.add_subgraph_stratified(
        "split",
        0,
        source_recv.flatten().pull_to_push().partition(
            |&name| name < "n",
            hf_builder.start_tee().map(Some).push_to(a_send),
            hf_builder.start_tee().map(Some).push_to(z_send),
        ),
    );
    hf_builder.add_subgraph_stratified(
        "merge",
        0,
        a_recv
            .flatten()
            .chain(z_recv.flatten())
            .pull_to_push()
            .map(Some)
            .push_to(stratum_boundary_send),
    );

    let mut hf = hf_builder.build();

    // Give BATCH_A and cross barrier to run next stratum.
    input.give(Iter(BATCH_A.iter().cloned()));
    input.flush();
    hf.tick_stratum();
    assert_eq!(None, output.get());
    hf.tick();
    assert_eq!(Some((BATCH_A.len(), "justin")), output.get());

    // Give BATCH_B but only run this stratum.
    input.give(Iter(BATCH_B.iter().cloned()));
    input.flush();
    hf.tick_stratum();

    // Give BATCH_C and run all to completion.
    input.give(Iter(BATCH_C.iter().cloned()));
    input.flush();
    hf.tick();
    // Second batch has 7+3 = 10 items.
    assert_eq!(Some((BATCH_B.len() + BATCH_C.len(), "mae")), output.get());
    assert_eq!(false, hf.next_stratum());
}
