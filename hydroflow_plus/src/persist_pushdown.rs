use crate::ir::*;

fn predicate_pushdown_node(node: HfPlusNode) -> HfPlusNode {
    match node {
        HfPlusNode::Map {
            f,
            input: box HfPlusNode::Persist(behind_persist),
        } => HfPlusNode::Persist(Box::new(predicate_pushdown_node(HfPlusNode::Map {
            f,
            input: behind_persist,
        }))),
        o => o,
    }
}

fn predicate_pushdown_leaf(leaf: HfPlusLeaf) -> HfPlusLeaf {
    match leaf {
        HfPlusLeaf::ForEach { f, input } => {
            let input = predicate_pushdown_node(*input);
            HfPlusLeaf::ForEach {
                f,
                input: Box::new(input),
            }
        }
        o => o,
    }
}

pub fn predicate_pushdown(ir: Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf> {
    ir.into_iter().map(predicate_pushdown_leaf).collect()
}

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

        let pushed_down = built.optimize_with(super::predicate_pushdown);

        insta::assert_debug_snapshot!(&pushed_down.ir);
    }
}
