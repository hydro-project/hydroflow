use crate::GraphType;
use hydroflow::scheduled::graph::Hydroflow;

pub fn print_graph(flow: &Hydroflow, graph: GraphType) {
    let meta_graph = flow
        .meta_graph()
        .expect("No graph found, maybe failed to parse.");
    match graph {
        GraphType::Mermaid => {
            println!("{}", meta_graph.to_mermaid(&Default::default()));
        }
        GraphType::Dot => {
            println!("{}", meta_graph.to_dot(&Default::default()))
        }
    }
}
