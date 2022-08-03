use hydroflow::hydroflow_syntax;
use tokio::task::LocalSet;

// TODO(mingwei): custom operators? How to handle in syntax? How to handle state?

// TODO(mingwei): Still need to handle crossing stratum boundaries
// TODO(mingwei): Implement non-monotonicity handling.

// TODO(mingwei): Tiemo user test after Tuesday.

// TODO(mingwei): Try to get more bad error messages to appear.

// TODO(joe): QOL: make a way to generate/print the mermaid graph.

// TODO(mingwei): Prevent unused variable warnings when hydroflow code is not generated.

// Joe:
// TODO(mingwei): Documentation articles.
// TODO(mingwei): Find a way to display join keys

#[test]
pub fn test_channel_minimal() {
    let (send, recv) = tokio::sync::mpsc::unbounded_channel::<usize>();

    let mut df1 = hydroflow_syntax! {
        recv_iter([1, 2, 3]) -> for_each(|x| { send.send(x).unwrap(); })
    };

    let mut df2 = hydroflow_syntax! {
        recv_stream(recv) -> for_each(|x| println!("{}", x))
    };

    df2.run_available();
    println!("A");
    df1.run_available();
    println!("B");
    df2.run_available();
}

#[test]
pub fn test_surface_syntax_reachability_generated() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        recv_iter(vec![0]) -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> tee();
        reached_vertices -> [0]my_join_tee;
        recv_stream(pairs_recv) -> [1]my_join_tee;

        my_join_tee[0] -> [1]reached_vertices;
        my_join_tee[1] -> for_each(|x| println!("Reached: {}", x));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
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
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        // edge(x,y) :- link(x,y)
        edge_merge_tee = merge() -> tee();
        link_tee = tee();
        recv_stream(pairs_recv) -> link_tee;
        link_tee[0] -> [0]edge_merge_tee;

        // edge(a,b) :- edge(a,k), link(k,b)
        the_join = join();
        edge_merge_tee[0] -> map(|(a, k)| (k, a)) -> [0]the_join;
        link_tee[1] -> [1]the_join;
        the_join -> map(|(_k, (a, b))| (a, b)) -> [1]edge_merge_tee;
        edge_merge_tee[1] -> for_each(|(a, b)| println!("transitive closure: ({},{})", a, b));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_dot()
    );

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

    // transitive closure: (0,1)
    // transitive closure: (2,4)
    // transitive closure: (3,4)
    // transitive closure: (1,2)
    // transitive closure: (0,2)
    // transitive closure: (1,4)
    // transitive closure: (0,4)
    // transitive closure: (0,3)
    // transitive closure: (0,4)
    // transitive closure: (0,3)
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
        contacts = recv_stream(contacts_recv) -> flat_map(|(pid_a, pid_b, time)| [(pid_a, (pid_b, time)), (pid_b, (pid_a, time))]);

        exposed = merge();
        recv_stream(diagnosed_recv) -> [0]exposed;

        new_exposed = (
            join() ->
            filter(|(_pid_a, ((_pid_b, t_contact), (t_from, t_to)))| {
                (t_from..=t_to).contains(&t_contact)
            }) ->
            map(|(_pid_a, (pid_b_t_contact, _t_from_to))| pid_b_t_contact) ->
            tee()
        );
        contacts -> [0]new_exposed;
        exposed -> [1]new_exposed;
        new_exposed[0] -> map(|(pid, t)| (pid, (t, t + TRANSMISSIBLE_DURATION))) -> [1]exposed;

        notifs = (
            join() ->
            for_each(|(_pid, ((name, phone), exposure))| {
                println!(
                    "[{}] To {}: Possible Exposure at t = {}",
                    name, phone, exposure
                );
            })
        );
        recv_stream(people_recv) -> [0]notifs;
        new_exposed[1] -> [1]notifs;
    };

    println!(
        "{}",
        hydroflow
            .serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_dot()
    );

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

#[test]
pub fn test_reduce() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        recv_iter(vec![0]) -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> tee();
        reached_vertices -> [0]my_join_tee;
        recv_stream(pairs_recv) -> [1]my_join_tee;

        my_join_tee[0] -> [1]reached_vertices;
        my_join_tee[1] -> reduce(|a, b| a + b) -> for_each(|sum| println!("{}", sum));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    println!("A");

    pairs_send.send((0, 1)).unwrap();
    pairs_send.send((2, 4)).unwrap();
    pairs_send.send((3, 4)).unwrap();
    pairs_send.send((1, 2)).unwrap();
    df.run_available();

    println!("B");

    pairs_send.send((0, 3)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    df.run_available();
}

#[tokio::test(flavor = "current_thread")]
async fn async_test() {
    LocalSet::new()
        .run_until(async {
            let (a_send, a_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();
            let (b_send, b_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();

            tokio::task::spawn_local(async move {
                let mut flow = hydroflow_syntax! {
                    recv_stream(a_recv) -> for_each(|x| { b_send.send(x).unwrap(); });
                };
                flow.run_async().await.unwrap();
            });
            tokio::task::spawn_local(async move {
                let mut flow = hydroflow_syntax! {
                    recv_stream(b_recv) -> for_each(|x| println!("{}", x));
                };
                flow.run_async().await.unwrap();
            });

            a_send.send(1).unwrap();
            a_send.send(2).unwrap();
            a_send.send(3).unwrap();

            tokio::task::yield_now().await;
        })
        .await;
}
