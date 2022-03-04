use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::handoff::VecHandoff;

const BATCH_A: [&'static str; 7] = ["megan", "davis", "mingwei", "john", "justin", "joe", "mae"];
const BATCH_B: [&'static str; 7] = [
    "mingwei", "lauren", "justin", "mae", "mingwei", "justin", "pierce",
];
const BATCH_C: [&'static str; 2] = ["joe", "mae"];

#[test]
fn groupby_core_monotonic() {
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

#[test]
#[ignore]
fn groupby_core_nonmon() {
    todo!("(mingwei): Requires strata.");
}
