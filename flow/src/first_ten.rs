use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten<'a, D: LocalDeploy<'a>>(
    graph: &'a GraphBuilder<'a, D>,
    node_builder: &impl NodeBuilder<'a, D>,
) {
    let node = graph.node(node_builder);

    let numbers = node.source_iter(q!(0..10));
    numbers.for_each(q!(|n| println!("{}", n)));
}

#[stageleft::entry]
pub fn first_ten_runtime<'a>(
    graph: &'a GraphBuilder<'a, SingleGraph>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    first_ten(graph, &());
    graph.build_single()
}
