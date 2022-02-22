use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

//This example detects size three cliques in a graph. Size three cliques are also known as triangles.
//The equivalent datalog program would be Triangle(x,y,z) := Edge(x,y), Edge(y,z), Edge(z,x)
pub fn main() {
    let mut builder = HydroflowBuilder::default();

    let (send_edges, recv_edges) =
        builder.add_channel_input::<_, _, VecHandoff<(usize, usize)>>("edge input");
    let (send_a, recv_a) = builder.make_edge::<_, VecHandoff<(usize, usize)>, _>("handoff_a"); 
    let (send_b, recv_b) = builder.make_edge::<_, VecHandoff<(usize, usize)>, _>("handoff_b");
    let (send_c, recv_c) = builder.make_edge::<_, VecHandoff<(usize, usize)>, _>("handoff_c");

    builder.add_subgraph(
        "teeing",
        recv_edges.flatten().pull_to_push().map(Some).tee(
            builder.start_tee().push_to(send_a),
            builder.start_tee().tee(
                builder.start_tee().push_to(send_b),
                builder.start_tee().push_to(send_c),
            ),
        ),
    );

    builder.add_subgraph(
        "joining",
        recv_a
            .flatten()
            .map(|(x, y)| (y, x))
            .join(recv_b.flatten())
            .map(|(y, x, z)| ((z, x), y)) //Here we have found all paths from x to z that go through y. Now we need to find edges that connect z back to x.
            .join(
                recv_c
                    .flatten()
                    .map(|(z, x)| ((z, x), ()))
            ).inspect(|&v| println!("three_clique found: {:?}", v))
            .pull_to_push()
            .for_each(|_| {}),
    );

    let mut hydroflow = builder.build();

    println!("A");

    send_edges.give(Some((5, 10)));
    send_edges.give(Some((0, 3)));
    send_edges.give(Some((3, 6)));
    send_edges.flush();
    hydroflow.tick();

    println!("B");

    send_edges.give(Some((6, 5)));
    send_edges.give(Some((6, 0))); //Creates a size three clique (triangle)
    send_edges.give(Some((10,6))); //Creates a size three clique (triangle)
    send_edges.flush();
    hydroflow.tick();
}
