use crate::{ir::*, RuntimeContext};
use stageleft::*;
use crate as hydroflow_plus;
use std::cell::RefCell;

pub fn increment_counter(tick: usize, id: u32, count: &mut u64) {
    *count += 1;
    println!("tick {}, id {}: counter {} incremented", tick, id, count);
}

fn quoted_any_fn<'a, F: Fn(usize) -> usize + 'a, Q: IntoQuotedMut<'a, F>>(q: Q) -> Q {
    q
}

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
        input: Box::new(child),
    }
}

pub fn profiling<'a>(ir: Vec<HfPlusLeaf>, context: RuntimeContext<'a>, counters: RuntimeData<&'a RefCell<Vec<u64>>>) -> Vec<HfPlusLeaf> {
    let mut id = 0;
    let mut seen_tees = Default::default();
    ir.into_iter().map(|l| l.transform_children( |node, seen_tees| add_profiling_node(node, context, counters, &mut id, seen_tees), &mut seen_tees)).collect()
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use stageleft::*;

    use crate::{Location, MultiGraph};

    #[test]
    fn predicate_pushdown_through_map() {
        let flow = crate::builder::FlowBuilder::<MultiGraph>::new();
        let process = flow.process(&());

        process
            .source_iter(q!(0..10))
            .all_ticks()
            .map(q!(|v| v + 1))
            .for_each(q!(|n| println!("{}", n)));

        let built = flow.build();

        insta::assert_debug_snapshot!(&built.ir);

        let pushed_down = built.optimize_with(|ir| super::profiling(ir, flow.runtime_context()));

        insta::assert_debug_snapshot!(&pushed_down.ir);
    }
}
