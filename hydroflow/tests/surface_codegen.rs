use hydroflow::hydroflow_syntax;

#[test]
pub fn test_surface_syntax_reachability_target() {
    use hydroflow::compiled::{IteratorToPusherator, PusheratorBuild};
    use hydroflow::scheduled::graph::Hydroflow;
    use hydroflow::scheduled::graph_ext::GraphExt;
    use hydroflow::scheduled::handoff::VecHandoff;
    use hydroflow::tl;

    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    let edges: HashMap<usize, Vec<usize>> = [
        (0, vec![1, 2, 3]),
        (1, vec![4, 5]),
        (2, vec![]),
        (4, vec![2]),
        (5, vec![1, 6, 7]),
        (6, vec![2]),
        (7, vec![10]),
        (8, vec![10]),
        (9, vec![10]),
        (10, vec![10]),
    ]
    .into_iter()
    .collect();

    // A dataflow that represents graph reachability.
    let mut df = Hydroflow::new();

    let (reachable_out, origins_in) = df.make_edge::<_, VecHandoff<usize>>("reachable -> origins");
    let (did_reach_out, possible_reach_in) =
        df.make_edge::<_, VecHandoff<usize>>("did_reach -> possible_reach");
    let (output_out, sink_in) = df.make_edge::<_, VecHandoff<usize>>("output -> sink");

    df.add_subgraph_source(
        "initially reachable source",
        reachable_out,
        move |_ctx, send| {
            send.give(Some(1));
        },
    );

    let seen_handle = df.add_state::<RefCell<HashSet<usize>>>(Default::default());

    df.add_subgraph(
        "main",
        tl!(origins_in, possible_reach_in),
        tl!(did_reach_out, output_out),
        move |context, tl!(origins, did_reach_recv), tl!(did_reach_send, output)| {
            let origins = origins.take_inner().into_iter();
            let possible_reach = did_reach_recv
                .take_inner()
                .into_iter()
                .filter_map(|v| edges.get(&v))
                .flatten()
                .copied();

            let mut seen_state = context.state_ref(seen_handle).borrow_mut();
            let pull = origins
                .chain(possible_reach)
                .filter(|v| seen_state.insert(*v));

            let pivot = pull
                .pull_to_push()
                .tee(hydroflow::compiled::for_each::ForEach::new(|v| {
                    did_reach_send.give(Some(v));
                }))
                .for_each(|v| {
                    output.give(Some(v));
                });

            pivot.run();
        },
    );

    let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
    let reachable_inner = reachable_verts.clone();
    df.add_subgraph_sink("output sink", sink_in, move |_ctx, recv| {
        (*reachable_inner).borrow_mut().extend(recv.take_inner());
    });

    df.run_available();

    println!("{:?}", *reachable_verts);
}

#[test]
pub fn test_surface_syntax_reachability_modified() {
    {
        use hydroflow::tl;
        let mut df = hydroflow::scheduled::graph::Hydroflow::new();
        let (hoff_9v1_send, hoff_9v1_recv) =
            df.make_edge::<_, hydroflow::scheduled::handoff::VecHandoff<_>>("handoff NodeId(9v1)");
        let (sg_1v1_node_7v1_send, mut sg_1v1_node_7v1_recv) =
            hydroflow::tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
        let mut sg_1v1_node_4v1_joindata = Default::default();
        df.add_subgraph(
            "Subgraph SubgraphId(1v1)",
            tl!(hoff_9v1_recv),
            tl!(hoff_9v1_send),
            move |context, tl!(hoff_9v1_recv), tl!(hoff_9v1_send)| {
                let hoff_9v1_recv = hoff_9v1_recv.take_inner().into_iter();
                let hoff_9v1_send = hydroflow::compiled::for_each::ForEach::new(|v| {
                    hoff_9v1_send.give(Some(v));
                });
                let op_3v1 = std::iter::IntoIterator::into_iter([0]);
                let op_1v1 = op_3v1.chain(hoff_9v1_recv);
                let op_2v1 = op_1v1.map(|v| (v, ()));
                let op_7v1 = {
                    std::iter::from_fn(|| {
                        match sg_1v1_node_7v1_recv
                            .poll_recv(&mut std::task::Context::from_waker(&mut context.waker()))
                        {
                            std::task::Poll::Ready(maybe) => maybe,
                            std::task::Poll::Pending => None,
                        }
                    })
                };
                let op_4v1 = hydroflow::compiled::pull::SymmetricHashJoin::new(
                    op_2v1,
                    op_7v1,
                    &mut sg_1v1_node_4v1_joindata,
                );
                let op_5v1 = op_4v1.map(|(_src, ((), dst))| dst);
                let op_8v1 = hydroflow::compiled::for_each::ForEach::new(|x| {
                    println!("Reached: {}\n", x);
                });
                let op_6v1 = hydroflow::compiled::tee::Tee::new(hoff_9v1_send, op_8v1);
                hydroflow::compiled::pivot::Pivot::new(op_5v1, op_6v1).run();
            },
        );
        df
    };
}

#[test]
pub fn test_surface_syntax_reachability_generated() {
    let (edges_in, edges_out) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = (merge() -> map(|v| (v, ())));
        (seed([0]) -> [0]reached_vertices);

        my_join = (join() -> map(|(_src, ((), dst))| dst) -> tee());
        (reached_vertices -> [0]my_join);
        (input(edges_out) -> [1]my_join);

        (my_join[0] -> [1]reached_vertices);
        (my_join[1] -> for_each(|x| println!("Reached: {}", x)));
    };

    df.run_available();

    edges_in.send((0, 1)).unwrap();
    df.run_available();

    edges_in.send((2, 4)).unwrap();
    edges_in.send((3, 4)).unwrap();
    df.run_available();

    edges_in.send((1, 2)).unwrap();
    df.run_available();

    edges_in.send((0, 3)).unwrap();
    df.run_available();

    edges_in.send((0, 3)).unwrap();
    df.run_available();

    // Reached: 1
    // Reached: 2
    // Reached: 4
    // Reached: 3
    // Reached: 4
}

#[test]
pub fn test_covid_tracing() {
    use tokio::sync::mpsc::unbounded_channel;

    const TRANSMISSIBLE_DURATION: usize = 14; // Days.

    type Pid = usize;
    type Name = &'static str;
    type Phone = &'static str;
    type DateTime = usize; // Days.

    let (contacts_send, contacts_recv) = unbounded_channel::<(Pid, Pid, DateTime)>();
    let (diagnosed_send, diagnosed_recv) = unbounded_channel::<(Pid, (DateTime, DateTime))>();
    let (people_send, people_recv) = unbounded_channel::<(Pid, (Name, Phone))>();

    let mut hydroflow = hydroflow_syntax! {
        looped = map(|(pid, t)| (pid, (t, t + TRANSMISSIBLE_DURATION)));
        contacts = (input(contacts_recv) -> flat_map(|(pid_a, pid_b, time)| [(pid_a, (pid_b, time)), (pid_b, (pid_a, time))]));

        exposed = (merge());
        (input(diagnosed_recv) -> exposed);
        (looped -> exposed);

        new_exposed = (
            join() ->
            filter(|(_pid_a, ((_pid_b, t_contact), (t_from, t_to)))| {
                (t_from..=t_to).contains(&t_contact)
            }) ->
            map(|(_pid_a, (pid_b_t_contact, _t_from_to))| pid_b_t_contact) ->
            tee()
        );
        (exposed -> [1]new_exposed);
        (contacts -> [0]new_exposed);
        (new_exposed -> looped);

        notifs = (
            join() ->
            for_each(|(_pid, ((name, phone), exposure))| {
                println!(
                    "[{}] To {}: Possible Exposure at t = {}",
                    name, phone, exposure
                );
            })
        );
        (input(people_recv) -> [0]notifs);
        (new_exposed -> [1]notifs);
    };

    {
        people_send
            .send((101, ("Mingwei S", "+1 650 555 7283")))
            .unwrap();
        people_send
            .send((102, ("Justin J", "+1 519 555 3458")))
            .unwrap();
        people_send
            .send((103, ("Mae M", "+1 912 555 9129")))
            .unwrap();

        contacts_send.send((101, 102, 1031)).unwrap(); // Mingwei + Justin
        contacts_send.send((101, 201, 1027)).unwrap(); // Mingwei + Joe

        let mae_diag_datetime = 1022;

        diagnosed_send
            .send((
                103, // Mae
                (
                    mae_diag_datetime,
                    mae_diag_datetime + TRANSMISSIBLE_DURATION,
                ),
            ))
            .unwrap();

        hydroflow.run_available();
        println!("A");

        contacts_send
            .send((101, 103, mae_diag_datetime + 6))
            .unwrap(); // Mingwei + Mae

        hydroflow.run_available();
        println!("B");

        people_send
            .send((103, ("Joe H", "+1 510 555 9999")))
            .unwrap();

        hydroflow.run_available();
    }
}
