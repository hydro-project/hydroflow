use clap::{Parser, ValueEnum};
use hydroflow::hydroflow_syntax;

#[derive(Parser, Debug, Clone, ValueEnum)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}
#[derive(Parser, Debug)]
struct Opts {
    #[clap(value_enum, long)]
    graph: Option<GraphType>,
}
pub fn main() {
    let opts = Opts::parse();
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        origin = source_iter(vec![0]);
        stream_of_edges = source_stream(pairs_recv) -> tee();
        reached_vertices = union()->tee();
        unreached_vertices = difference();
        origin -> [0]reached_vertices;

        all_vertices = stream_of_edges[0]
          -> flat_map(|(src, dst)| [src, dst])
          -> tee();

        // the join for reachable nodes
        my_join = join() -> flat_map(|(src, ((), dst))| [src, dst]);
        reached_vertices[0] -> map(|v| (v, ())) -> [0]my_join;
        stream_of_edges[1] -> [1]my_join;

        // the loop
        my_join -> [1]reached_vertices;

        // the difference all_vertices - reached_vertices
        all_vertices[0] -> [pos]unreached_vertices;
        reached_vertices[1] -> [neg]unreached_vertices;

        // the output
        all_vertices[1] -> for_each(|v| println!("Received vertex: {}", v));
        unreached_vertices -> for_each(|v| println!("unreached_vertices vertex: {}", v));
    };

    if let Some(graph) = opts.graph {
        let serde_graph = df
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    println!("A");

    pairs_send.send((5, 10)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    pairs_send.send((3, 6)).unwrap();
    // df.run_available();

    println!("B");
    pairs_send.send((6, 5)).unwrap();
    pairs_send.send((11, 12)).unwrap();
    df.run_available();

    // A
    // Received vertex: 6
    // Received vertex: 10
    // Received vertex: 3
    // Received vertex: 5
    // Received vertex: 0
    // unreached_vertices vertex: 10
    // unreached_vertices vertex: 5
    // B
    // Received vertex: 11
    // Received vertex: 6
    // Received vertex: 5
    // Received vertex: 12
    // unreached_vertices vertex: 11
    // unreached_vertices vertex: 12
}
