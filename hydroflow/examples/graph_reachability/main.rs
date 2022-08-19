use clap::{ArgEnum, Parser};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

#[derive(Clone, ArgEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}
#[derive(Parser, Debug)]
struct Opts {
    #[clap(arg_enum, long)]
    graph: GraphType,
}
pub fn main() {
    let opts = Opts::parse();
    let mut builder = HydroflowBuilder::default();

    let (send_edges, recv_edges) =
        builder.add_channel_input::<_, _, VecHandoff<(usize, usize)>>("edge input");
    let (send_loop, recv_loop) = builder.make_edge::<_, VecHandoff<usize>, _>("loop");

    builder.add_subgraph(
        "main",
        std::iter::once(0)
            .into_hydroflow()
            .chain(recv_loop.flatten())
            .map(|v| (v, ()))
            .join(recv_edges.flatten())
            .pull_to_push()
            .map(|(_old_v, ((), new_v))| new_v)
            .tee(
                builder.start_tee().for_each(|v| println!("Reached: {}", v)),
                builder.start_tee().map(Some).push_to(send_loop),
            ),
    );

    let mut hf = builder.build();
    match opts.graph {
        GraphType::Mermaid => {
            println!("{}", hf.generate_mermaid())
        }
        GraphType::Dot => {
            println!("{}", hf.generate_dot())
        }
        GraphType::Json => {
            println!("{}", hf.generate_json())
        }
    }

    println!("A");

    send_edges.give(Some((5, 10)));
    send_edges.give(Some((0, 3)));
    send_edges.give(Some((3, 6)));
    send_edges.flush();
    hf.run_available();

    println!("B");

    send_edges.give(Some((6, 5)));
    send_edges.flush();
    hf.run_available();
}
