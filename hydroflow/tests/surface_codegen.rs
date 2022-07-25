use hydroflow::{hydroflow_parser, hydroflow_syntax};

// TODO(mingwei): error message for ownership duplicate
// (input(edges_out) -> [0]my_join_tee);
// (input(edges_out) -> [1]my_join_tee);

// TODO(mingwei): remove automatic index counting

// TODO(mingwei): custom operators? How to handle in syntax? How to handle state?

// TODO(mingwei): Better name for `input(...)`
// TODO(mingwei): Rename `seed`, really converts a Rust iterator to hydroflow pipeline.

// TODO(mingwei): Still need to handle crossing stratum boundaries

// TODO(mingwei): Tiemo user test after Tuesday.

// TODO(mingwei): Try to get more bad error messages to appear.

// TODO(mingwei): QOL: make a way to generate/print the mermaid graph.

// TODO(mingwei): Implement non-monotonicity handling.

// TODO(mingwei): Prevent unused variable warnings when hydroflow code is not generated.

// Joe:
// TODO(mingwei): Documentation articles.
// TODO(mingwei): Rename `hydroflow_lang` -> `hydroflow_lang`

#[test]
pub fn test_surface_syntax_reachability_generated() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = (merge() -> map(|v| (v, ())));
        (seed(vec![0]) -> [0]reached_vertices);

        my_join_tee = (join() -> map(|(_src, ((), dst))| dst) -> tee());
        (reached_vertices -> [0]my_join_tee);
        (input(pairs_recv) -> [1]my_join_tee);

        (my_join_tee[0] -> [1]reached_vertices);
        (my_join_tee[1] -> for_each(|x| println!("Reached: {}", x)));
    };

    df.run_available();

    pairs_send.send((0, 1)).unwrap();
    df.run_available();

    pairs_send.send((2, 4)).unwrap();
    pairs_send.send((3, 4)).unwrap();
    df.run_available();

    pairs_send.send((1, 2)).unwrap();
    df.run_available();

    pairs_send.send((0, 3)).unwrap();
    df.run_available();

    pairs_send.send((0, 3)).unwrap();
    df.run_available();

    // Reached: 1
    // Reached: 2
    // Reached: 4
    // Reached: 3
    // Reached: 4
}

#[test]
pub fn test_transitive_closure() {
    // WIP

    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = (merge() -> map(|v| (v, ())));

        input_tee = tee();
        node_merge = merge();
        (input(pairs_recv) -> input_tee);
        (input_tee[0] -> map(|v: (usize, usize)| v.0) -> [0]node_merge);
        (input_tee[1] -> map(|v: (usize, usize)| v.1) -> [1]node_merge);
        (node_merge -> [0]reached_vertices);

        my_join_tee = (join() -> map(|(_src, ((), dst))| dst) -> tee());
        (reached_vertices -> [0]my_join_tee);
        (input_tee[2] -> [1]my_join_tee);

        (my_join_tee[0] -> [1]reached_vertices);
        (my_join_tee[1] -> for_each(|x| println!("Reached: {}", x)));
    };

    df.run_available();

    pairs_send.send((0, 1)).unwrap();
    df.run_available();

    pairs_send.send((2, 4)).unwrap();
    pairs_send.send((3, 4)).unwrap();
    df.run_available();

    pairs_send.send((1, 2)).unwrap();
    df.run_available();

    pairs_send.send((0, 3)).unwrap();
    df.run_available();

    pairs_send.send((0, 3)).unwrap();
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
        contacts = (input(contacts_recv) -> flat_map(|(pid_a, pid_b, time)| [(pid_a, (pid_b, time)), (pid_b, (pid_a, time))]));

        exposed = merge();
        (input(diagnosed_recv) -> [0]exposed);

        new_exposed = (
            join() ->
            filter(|(_pid_a, ((_pid_b, t_contact), (t_from, t_to)))| {
                (t_from..=t_to).contains(&t_contact)
            }) ->
            map(|(_pid_a, (pid_b_t_contact, _t_from_to))| pid_b_t_contact) ->
            tee()
        );
        (contacts -> [0]new_exposed);
        (exposed -> [1]new_exposed);
        (new_exposed[0] -> map(|(pid, t)| (pid, (t, t + TRANSMISSIBLE_DURATION))) -> [1]exposed);

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
        (new_exposed[1] -> [1]notifs);
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
