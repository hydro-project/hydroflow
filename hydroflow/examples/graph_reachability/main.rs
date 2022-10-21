use clap::{ArgEnum, Parser};
use hydroflow::hydroflow_syntax;

#[derive(Parser, Debug, Clone, ArgEnum)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}
#[derive(Parser, Debug)]
struct Opts {
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}
pub fn main() {
    let opts = Opts::parse();
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (edges_send, edges_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        recv_iter(vec![0]) -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> tee();
        reached_vertices -> [0]my_join_tee;
        recv_stream(edges_recv) -> [1]my_join_tee;

        my_join_tee[0] -> [1]reached_vertices;
        my_join_tee[1] -> for_each(|x| println!("Reached: {}", x));
    };

    if let Some(graph) = opts.graph {
        let serde_graph = df
            .serde_graph()
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

    df.run_available();

    println!("A");

    edges_send.send((5, 10)).unwrap();
    edges_send.send((0, 3)).unwrap();
    edges_send.send((3, 6)).unwrap();
    df.run_available();

    println!("B");

    edges_send.send((6, 5)).unwrap();
    df.run_available();

    // A
    // Reached: 3
    // Reached: 6
    // B
    // Reached: 5
    // Reached: 10
}
