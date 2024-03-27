<<<<<<< HEAD
use crate::{ir::*, RuntimeContext};
use stageleft::*;
use crate as hydroflow_plus;
use std::cell::RefCell;

pub fn increment_counter(tick: usize, id: u32, count: &mut u64) {
    *count += 1;
    println!("tick {}, id {}: counter {} incremented", tick, id, count);
=======
use std::cell::RefCell;

use hydroflow::futures::channel::mpsc::UnboundedSender;
use stageleft::*;

use crate as hydroflow_plus;
use crate::ir::*;
use crate::RuntimeContext;

pub fn increment_counter(tick: usize, id: u32, count: &mut u64) {
    *count += 1;
    // println!("tick {}, id {}: counter {} incremented", tick, id, count);
>>>>>>> ee4e33c001 (Profiling counts can be processed outside of HF)
}

fn quoted_any_fn<'a, F: Fn(usize) -> usize + 'a, Q: IntoQuotedMut<'a, F>>(q: Q) -> Q {
    q
}

<<<<<<< HEAD
fn add_profiling_node<'a>(node: HfPlusNode, context: RuntimeContext<'a>, counters: RuntimeData<&'a RefCell<Vec<u64>>>, id: &mut u32, seen_tees: &mut SeenTees) -> HfPlusNode {
    let my_id = *id;
    *id += 1;

    let child = node.transform_children(|node, seen_tees| add_profiling_node(node, context, counters, id, seen_tees), seen_tees);
    let counters_copy = *&counters;
    HfPlusNode::Map {
        f: quoted_any_fn(q!(
            {
            counters_copy.borrow_mut()[my_id as usize] = 0;
            move |v| { hydroflow_plus::profiler::increment_counter(context.current_tick(), my_id, &mut counters.borrow_mut()[my_id as usize]); v }
        })).splice().into(),
=======
// Add a profiling node before each node to count the cardinality of its input
fn add_profiling_node<'a>(
    node: HfPlusNode,
    context: RuntimeContext<'a>,
    counters: RuntimeData<&'a RefCell<Vec<u64>>>,
    counter_queue: RuntimeData<&'a RefCell<UnboundedSender<(usize, u64)>>>,
    id: &mut u32,
    seen_tees: &mut SeenTees,
) -> HfPlusNode {
    let my_id = *id;
    *id += 1;

    let child = node.transform_children(
        |node, seen_tees| add_profiling_node(node, context, counters, counter_queue, id, seen_tees),
        seen_tees,
    );
    let counters_copy1 = *&counters;
    let counters_copy2 = *&counters;
    let my_id_copy1 = my_id;
    let my_id_copy2 = my_id;
    let my_id_copy3 = my_id;
    let my_id_copy4 = my_id;
    HfPlusNode::Map {
        f: quoted_any_fn(q!({
            // Put counters on queue
            counter_queue.borrow().unbounded_send((my_id_copy1 as usize, counters_copy1.borrow()[my_id_copy2 as usize])).unwrap();
            counters_copy2.borrow_mut()[my_id_copy3 as usize] = 0;
            move |v| {
                hydroflow_plus::profiler::increment_counter(
                    context.current_tick(),
                    my_id_copy4,
                    &mut counters.borrow_mut()[my_id as usize],
                );
                v
            }
        }))
        .splice()
        .into(),
>>>>>>> ee4e33c001 (Profiling counts can be processed outside of HF)
        input: Box::new(child),
    }
}

<<<<<<< HEAD
pub fn profiling<'a>(ir: Vec<HfPlusLeaf>, context: RuntimeContext<'a>, counters: RuntimeData<&'a RefCell<Vec<u64>>>) -> Vec<HfPlusLeaf> {
    let mut id = 0;
    let mut seen_tees = Default::default();
    ir.into_iter().map(|l| l.transform_children( |node, seen_tees| add_profiling_node(node, context, counters, &mut id, seen_tees), &mut seen_tees)).collect()
=======
// Count the cardinality of each input and periodically output to a file
pub fn profiling<'a>(
    ir: Vec<HfPlusLeaf>,
    context: RuntimeContext<'a>,
    counters: RuntimeData<&'a RefCell<Vec<u64>>>,
    counter_queue: RuntimeData<&'a RefCell<UnboundedSender<(usize, u64)>>>,
) -> Vec<HfPlusLeaf> {
    let mut id = 0;
    let mut seen_tees = Default::default();
    ir.into_iter()
        .map(|l| {
            l.transform_children(
                |node, seen_tees| add_profiling_node(node, context, counters, counter_queue, &mut id, seen_tees),
                &mut seen_tees,
            )
        })
        .collect()
>>>>>>> ee4e33c001 (Profiling counts can be processed outside of HF)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
<<<<<<< HEAD
    use stageleft::*;

    use crate::{Location, MultiGraph};
    use hydroflow_lang::graph::WriteConfig;
=======
    use hydroflow_lang::graph::WriteConfig;
    use stageleft::*;

    use crate::{Location, MultiGraph};
>>>>>>> ee4e33c001 (Profiling counts can be processed outside of HF)

    #[test]
    fn predicate_pushdown_through_map() {
        let flow = crate::builder::FlowBuilder::<MultiGraph>::new();
        let process = flow.process(&());

        process
            .source_iter(q!(0..10))
            .all_ticks()
            .map(q!(|v| v + 1))
            .for_each(q!(|n| println!("{}", n)));

<<<<<<< HEAD
=======
        let runtime_context = flow.runtime_context();
>>>>>>> ee4e33c001 (Profiling counts can be processed outside of HF)
        let built = flow.extract();

        insta::assert_debug_snapshot!(&built.ir);

        // Print mermaid
<<<<<<< HEAD
        let mut mermaid_config = WriteConfig {op_text_no_imports: true, ..Default::default()};
        for (_, ir) in built.clone().optimize_default().hydroflow_ir() {
            println!("{}", ir.to_mermaid(&mermaid_config));
        }

        let counters = RuntimeData::new("Fake");
        let pushed_down = built.optimize_with(|ir| super::profiling(ir, flow.runtime_context(), counters));
=======
        // let mut mermaid_config = WriteConfig {op_text_no_imports: true, ..Default::default()};
        // for (_, ir) in built.clone().optimize_default().hydroflow_ir() {
        //     println!("{}", ir.to_mermaid(&mermaid_config));
        // }

        let counters = RuntimeData::new("Fake");
        let counter_queue = RuntimeData::new("Fake");
        
        let pushed_down =
            built.optimize_with(|ir| super::profiling(ir, runtime_context, counters, counter_queue));
>>>>>>> ee4e33c001 (Profiling counts can be processed outside of HF)

        insta::assert_debug_snapshot!(&pushed_down.ir);
    }
}
