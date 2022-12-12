use std::collections::{HashMap, HashSet};

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::recv_into;

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
pub fn test_basic_2() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([1]) -> for_each(|v| out_send.send(v).unwrap());
    };
    df.run_available();

    assert_eq!(&[1], &*recv_into::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_basic_3() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([1]) -> map(|v| v + 1) -> for_each(|v| out_send.send(v).unwrap());
    };
    df.run_available();

    assert_eq!(&[2], &*recv_into::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_basic_merge() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        m = merge() -> for_each(|v| out_send.send(v).unwrap());
        source_iter([1]) -> [0]m;
        source_iter([2]) -> [1]m;
    };
    df.run_available();

    assert_eq!(&[1, 2], &*recv_into::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_basic_tee() {
    let (out_send_a, mut out_recv) = hydroflow::util::unbounded_channel::<String>();
    let out_send_b = out_send_a.clone();

    let mut df = hydroflow_syntax! {
        t = source_iter([1]) -> tee();
        t[0] -> for_each(|v| out_send_a.send(format!("A {}", v)).unwrap());
        t[1] -> for_each(|v| out_send_b.send(format!("B {}", v)).unwrap());
    };
    df.run_available();

    let out: HashSet<_> = recv_into(&mut out_recv);
    assert_eq!(2, out.len());
    assert!(out.contains(&"A 1".to_owned()));
    assert!(out.contains(&"B 1".to_owned()));
}

#[test]
pub fn test_basic_inspect_null() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let seen = Rc::new(RefCell::new(Vec::new()));
    let seen_inner = Rc::clone(&seen);

    let mut df = hydroflow_syntax! {
        source_iter([1, 2, 3, 4]) -> inspect(|&x| seen_inner.borrow_mut().push(x)) -> null();
    };
    df.run_available();

    assert_eq!(&[1, 2, 3, 4], &**seen.borrow());
}

// Mainly checking subgraph partitioning pull-push handling.
#[test]
pub fn test_large_diamond() {
    #[allow(clippy::map_identity)]
    let mut df: Hydroflow = hydroflow_syntax! {
        t = source_iter([1]) -> tee();
        j = merge() -> for_each(|x| println!("{}", x));
        t[0] -> map(std::convert::identity) -> map(std::convert::identity) -> [0]j;
        t[1] -> map(std::convert::identity) -> map(std::convert::identity) -> [1]j;
    };
    df.run_available();
}

/// Test that source_stream can handle "complex" expressions.
#[test]
pub fn test_recv_expr() {
    let send_recv = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_stream(send_recv.1)
            -> for_each(|v| print!("{:?}", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    let items_send = send_recv.0;
    items_send.send(9).unwrap();
    items_send.send(2).unwrap();
    items_send.send(5).unwrap();
    df.run_available();
}

#[test]
pub fn test_unzip() {
    let (send0, mut recv0) = hydroflow::util::unbounded_channel::<&'static str>();
    let (send1, mut recv1) = hydroflow::util::unbounded_channel::<&'static str>();
    let mut df = hydroflow_syntax! {
        my_unzip = source_iter(vec![("Hello", "Foo"), ("World", "Bar")]) -> unzip();
        my_unzip[0] -> for_each(|v| send0.send(v).unwrap());
        my_unzip[1] -> for_each(|v| send1.send(v).unwrap());
    };

    df.run_available();

    let out0: Vec<_> = recv_into(&mut recv0);
    assert_eq!(&["Hello", "World"], &*out0);
    let out1: Vec<_> = recv_into(&mut recv1);
    assert_eq!(&["Foo", "Bar"], &*out1);
}

#[test]
pub fn test_join_order() {
    let _df_good = hydroflow_syntax! {
        yikes = join() -> for_each(|m: ((), (u32, String))| println!("{:?}", m));
        source_iter([0,1,2]) -> map(|i| ((), i)) -> [0]yikes;
        source_iter(["a".to_string(),"b".to_string(),"c".to_string()]) -> map(|s| ((), s)) -> [1]yikes;
    };
    let _df_bad = hydroflow_syntax! {
        yikes = join() -> for_each(|m: ((), (u32, String))| println!("{:?}", m));
        source_iter(["a".to_string(),"b".to_string(),"c".to_string()]) -> map(|s| ((), s)) -> [1]yikes;
        source_iter([0,1,2]) -> map(|i| ((), i)) -> [0]yikes;
    };
}

#[test]
pub fn test_cross_join() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, &str)>();

    let mut df = hydroflow_syntax! {
        cj = cross_join() -> for_each(|v| out_send.send(v).unwrap());
        source_iter([1, 2, 3]) -> [0]cj;
        source_iter(["a", "b", "c"]) -> [1]cj;
    };
    df.run_available();

    let out: HashSet<_> = recv_into(&mut out_recv);
    assert_eq!(3 * 3, out.len());
    for n in [1, 2, 3] {
        for c in ["a", "b", "c"] {
            assert!(out.contains(&(n, c)));
        }
    }
}

#[test]
pub fn test_flatten() {
    // test pull
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(u8, u8)>();
    let mut df_pull = hydroflow_syntax! {
        source_iter([(1,1), (1,2), (2,3), (2,4)])
        -> fold(HashMap::<u8,u8>::new(), |mut ht, t:(u8,u8)| {
                let e = ht.entry(t.0).or_insert(0);
                *e += t.1;
                ht})
        -> flatten()
        -> for_each(|(k,v)| out_send.send((k,v)).unwrap());
    };
    df_pull.run_available();

    let out: HashSet<_> = recv_into(&mut out_recv);
    for pair in [(1, 3), (2, 7)] {
        assert!(out.contains(&pair));
    }

    // test push
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(u8, u8)>();
    let mut df_push = hydroflow_syntax! {
        datagen = source_iter([(1,2), (1,2), (2,4), (2,4)]) -> tee();
        datagen[0] -> fold(HashMap::<u8,u8>::new(), |mut ht, t:(u8,u8)| {
                let e = ht.entry(t.0).or_insert(0);
                *e += t.1;
                ht})
        -> flatten()
        -> for_each(|(k,v)| out_send.send((k,v)).unwrap());
        datagen[1] -> null();
    };

    df_push.run_available();

    let out: HashSet<_> = recv_into(&mut out_recv);
    for pair in [(1, 4), (2, 8)] {
        assert!(out.contains(&pair));
    }
}

#[test]
pub fn test_next_epoch() {
    let (inp_send, inp_recv) = hydroflow::util::unbounded_channel::<usize>();
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();
    let mut flow = hydroflow::hydroflow_syntax! {
        inp = source_stream(inp_recv) -> tee();
        diff = difference() -> for_each(|x| out_send.send(x).unwrap());
        inp -> [pos]diff;
        inp -> next_epoch() -> [neg]diff;
    };

    for x in [1, 2, 3, 4] {
        inp_send.send(x).unwrap();
    }
    flow.run_epoch();

    for x in [3, 4, 5, 6] {
        inp_send.send(x).unwrap();
    }
    flow.run_epoch();

    flow.run_available();
    let out: Vec<_> = recv_into(&mut out_recv);
    assert_eq!(&[1, 2, 3, 4, 5, 6], &*out);
}

#[test]
pub fn test_reduce_sum() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_stream(items_recv)
            -> reduce(|a, b| a + b)
            -> for_each(|v| print!("{:?}", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    print!("\nA: ");

    items_send.send(9).unwrap();
    items_send.send(2).unwrap();
    items_send.send(5).unwrap();
    df.run_available();

    print!("\nB: ");

    items_send.send(9).unwrap();
    items_send.send(5).unwrap();
    items_send.send(2).unwrap();
    items_send.send(0).unwrap();
    items_send.send(3).unwrap();
    df.run_available();

    println!();
}

#[test]
pub fn test_sort() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_stream(items_recv)
            -> sort()
            -> for_each(|v| print!("{:?}, ", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    print!("\nA: ");

    items_send.send(9).unwrap();
    items_send.send(2).unwrap();
    items_send.send(5).unwrap();
    df.run_available();

    print!("\nB: ");

    items_send.send(9).unwrap();
    items_send.send(5).unwrap();
    items_send.send(2).unwrap();
    items_send.send(0).unwrap();
    items_send.send(3).unwrap();
    df.run_available();

    println!();
}

#[test]
pub fn test_unique() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_stream(items_recv)
            -> unique()
            -> for_each(|v| print!("{:?}, ", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    print!("\nA: ");

    items_send.send(9).unwrap();
    items_send.send(9).unwrap();
    items_send.send(5).unwrap();
    df.run_available();

    print!("\nB: ");

    items_send.send(9).unwrap();
    items_send.send(9).unwrap();
    items_send.send(2).unwrap();
    items_send.send(0).unwrap();
    items_send.send(2).unwrap();
    df.run_available();

    println!();
}

#[test]
pub fn test_fold_sort() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_stream(items_recv)
            -> fold(Vec::new(), |mut v, x| {
                v.push(x);
                v
            })
            -> flat_map(|mut vec| { vec.sort(); vec })
            -> for_each(|v| print!("{:?}, ", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    print!("\nA: ");

    items_send.send(9).unwrap();
    items_send.send(2).unwrap();
    items_send.send(5).unwrap();
    df.run_available();

    print!("\nB: ");

    items_send.send(9).unwrap();
    items_send.send(5).unwrap();
    items_send.send(2).unwrap();
    items_send.send(0).unwrap();
    items_send.send(3).unwrap();
    df.run_available();

    println!();
}

#[test]
pub fn test_groupby() {
    let (items_send, items_recv) = hydroflow::util::unbounded_channel::<(u32, Vec<u32>)>();

    let mut df = hydroflow_syntax! {
        source_stream(items_recv)
            -> groupby(Vec::new, |old: &mut Vec<u32>, mut x: Vec<u32>| old.append(&mut x))
            -> for_each(|v| print!("{:?}, ", v));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    items_send.send((0, vec![1, 2])).unwrap();
    items_send.send((0, vec![3, 4])).unwrap();
    items_send.send((1, vec![1])).unwrap();
    items_send.send((1, vec![1, 2])).unwrap();
    df.run_available();

    println!();
}

#[test]
pub fn test_demux_1() {
    enum Shape {
        Circle(f64),
        Rectangle { width: f64, height: f64 },
        Square(f64),
    }

    let mut df = hydroflow_syntax! {
        my_demux = source_iter([
            Shape::Circle(5.0),
            Shape::Rectangle { width: 10.0, height: 8.0 },
            Shape::Square(9.0),
        ]) -> demux(|shape, var_args!(circ, rect)| {
            match shape {
                Shape::Circle(radius) => circ.give(radius),
                Shape::Rectangle { width, height } => rect.give((width, height)),
                Shape::Square(side) => rect.give((side, side)),
            }
        });

        out = merge() -> for_each(|a| println!("area: {}", a));

        my_demux[circ] -> map(|r| std::f64::consts::PI * r * r) -> out;
        my_demux[rect] -> map(|(w, h)| w * h) -> out;
    };
    df.run_available();
}

#[test]
pub fn test_demux_fizzbuzz_1() {
    let mut df = hydroflow_syntax! {
        my_demux = source_iter(1..=100)
            -> demux(|v, var_args!(fzbz, fizz, buzz, vals)|
                match v {
                    v if 0 == v % 15 => fzbz.give(()),
                    v if 0 == v % 3 => fizz.give(()),
                    v if 0 == v % 5 => buzz.give(()),
                    v => vals.give(v),
                }
            );
        my_demux[fzbz] -> for_each(|_| println!("fizzbuzz"));
        my_demux[fizz] -> for_each(|_| println!("fizz"));
        my_demux[buzz] -> for_each(|_| println!("buzz"));
        my_demux[vals] -> for_each(|x| println!("{}", x));
    };
    df.run_available();
}

#[test]
pub fn test_demux_fizzbuzz_2() {
    let mut df = hydroflow_syntax! {
        my_demux = source_iter(1..=100)
        -> demux(|v, var_args!(fzbz, fizz, buzz, vals)|
            match (v % 3, v % 5) {
                (0, 0) => fzbz.give(()),
                (0, _) => fizz.give(()),
                (_, 0) => buzz.give(()),
                (_, _) => vals.give(v),
            }
        );
        my_demux[fzbz] -> for_each(|_| println!("fizzbuzz"));
        my_demux[fizz] -> for_each(|_| println!("fizz"));
        my_demux[buzz] -> for_each(|_| println!("buzz"));
        my_demux[vals] -> for_each(|x| println!("{}", x));
    };
    df.run_available();
}

#[test]
pub fn test_channel_minimal() {
    let (send, recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df1 = hydroflow_syntax! {
        source_iter([1, 2, 3]) -> for_each(|x| { send.send(x).unwrap(); })
    };

    let mut df2 = hydroflow_syntax! {
        source_stream(recv) -> for_each(|x| println!("{}", x))
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
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut df: Hydroflow = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        source_iter(vec![0]) -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> tee();
        reached_vertices -> [0]my_join_tee;
        source_stream(pairs_recv) -> [1]my_join_tee;

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
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        // edge(x,y) :- link(x,y)
        edge_merge_tee = merge() -> tee();
        link_tee = tee();
        source_stream(pairs_recv) -> link_tee;
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
    use hydroflow::util::unbounded_channel;

    const TRANSMISSIBLE_DURATION: usize = 14; // Days.

    type Pid = usize;
    type Name = &'static str;
    type Phone = &'static str;
    type DateTime = usize; // Days.

    let (contacts_send, contacts_recv) = unbounded_channel::<(Pid, Pid, DateTime)>();
    let (diagnosed_send, diagnosed_recv) = unbounded_channel::<(Pid, (DateTime, DateTime))>();
    let (people_send, people_recv) = unbounded_channel::<(Pid, (Name, Phone))>();

    let mut hydroflow = hydroflow_syntax! {
        contacts = source_stream(contacts_recv) -> flat_map(|(pid_a, pid_b, time)| [(pid_a, (pid_b, time)), (pid_b, (pid_a, time))]);

        exposed = merge();
        source_stream(diagnosed_recv) -> [0]exposed;

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
        source_stream(people_recv) -> [0]notifs;
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

/// This tests graph reachability along with an accumulation (in this case sum of vertex ids).
/// This is to test fixed-point being reched before the accumulation running.
#[test]
pub fn test_reduce() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        source_iter(vec![0]) -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> tee();
        reached_vertices -> [0]my_join_tee;
        source_stream(pairs_recv) -> [1]my_join_tee;

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

use serde::{Deserialize, Serialize};
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct UsizeMessage {
    payload: usize,
}
#[test]
pub fn simple_test() {
    // Create our channel input
    let (input_example, example_recv) = hydroflow::util::unbounded_channel::<UsizeMessage>();

    let mut flow = hydroflow_syntax! {
        source_stream(example_recv)
        -> filter_map(|n: UsizeMessage| {
            let n2 = n.payload * n.payload;
            if n2 > 10 {
                Some(n2)
            }
            else {
                None
            }
        })
        -> flat_map(|n| (n..=n+1))
        -> for_each(|n| println!("Ahoj {}", n))
    };

    println!("A");
    input_example.send(UsizeMessage { payload: 1 }).unwrap();
    input_example.send(UsizeMessage { payload: 0 }).unwrap();
    input_example.send(UsizeMessage { payload: 2 }).unwrap();
    input_example.send(UsizeMessage { payload: 3 }).unwrap();
    input_example.send(UsizeMessage { payload: 4 }).unwrap();
    input_example.send(UsizeMessage { payload: 5 }).unwrap();

    flow.run_available();

    println!("B");
    input_example.send(UsizeMessage { payload: 6 }).unwrap();
    input_example.send(UsizeMessage { payload: 7 }).unwrap();
    input_example.send(UsizeMessage { payload: 8 }).unwrap();
    input_example.send(UsizeMessage { payload: 9 }).unwrap();
    flow.run_available();
}
