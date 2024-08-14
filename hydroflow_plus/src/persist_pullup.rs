use crate::ir::*;

fn persist_pullup_node<'a>(node: HfPlusNode<'a>, seen_tees: &mut SeenTees<'a>) -> HfPlusNode<'a> {
    match node.transform_children(persist_pullup_node, seen_tees) {
        HfPlusNode::Map {
            f,
            input: box HfPlusNode::Persist(behind_persist),
        } => HfPlusNode::Persist(Box::new(HfPlusNode::Map {
            f,
            input: behind_persist,
        })),
        o => o,
    }
}

pub fn persist_pullup(ir: Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf> {
    let mut seen_tees = Default::default();
    ir.into_iter()
        .map(|l| l.transform_children(persist_pullup_node, &mut seen_tees))
        .collect()
}

#[cfg(test)]
mod tests {
    use stageleft::*;

    use crate::MultiGraph;

    #[test]
    fn persist_pullup_through_map() {
        let flow = crate::builder::FlowBuilder::<MultiGraph>::new();
        let process = flow.process(());

        flow.source_iter(&process, q!(0..10))
            .all_ticks()
            .map(q!(|v| v + 1))
            .for_each(q!(|n| println!("{}", n)));

        let built = flow.finalize();

        insta::assert_debug_snapshot!(built.ir());

        let optimized = built.optimize_with(super::persist_pullup);

        insta::assert_debug_snapshot!(optimized.ir());
        for (id, graph) in optimized.compile().hydroflow_ir() {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_display_snapshot!(graph.surface_syntax_string());
            });
        }
    }

    #[test]
    fn persist_pullup_behind_tee() {
        let flow = crate::builder::FlowBuilder::<MultiGraph>::new();
        let process = flow.process(());

        let before_tee = flow
            .source_iter(&process, q!(0..10))
            .all_ticks()
            .map(q!(|v| v + 1));

        before_tee.clone().for_each(q!(|n| println!("{}", n)));

        before_tee.for_each(q!(|n| println!("{}", n)));

        let built = flow.finalize();

        insta::assert_debug_snapshot!(built.ir());

        let optimized = built.optimize_with(super::persist_pullup);

        insta::assert_debug_snapshot!(optimized.ir());

        for (id, graph) in optimized.compile().hydroflow_ir() {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_display_snapshot!(graph.surface_syntax_string());
            });
        }
    }
}
