use std::thread;

use datalog_compiler::datalog;

#[tokio::test]
pub async fn test_minimal() {
    let (in_send, input) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input input
            .output out
    
            out(y, x) :- input(x, y).
            "#
        );

        flow.run_available();
    })
    .join()
    .unwrap();

    assert_eq!(out_recv.recv().await.unwrap(), (2, 1));
    assert_eq!(out_recv.recv().await, None);
}

#[tokio::test]
pub async fn test_join_with_self() {
    let (in_send, input) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();
    in_send.send((1, 3)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input input
            .output out

            out(x, y) :- input(x, y), input(y, x).
            "#
        );

        flow.run_available();
    })
    .join()
    .unwrap();

    assert_eq!(out_recv.recv().await.unwrap(), (2, 1));
    assert_eq!(out_recv.recv().await.unwrap(), (1, 2));
    assert_eq!(out_recv.recv().await, None);
}

#[tokio::test]
pub async fn test_multi_use_intermediate() {
    let (in_send, input) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();
    in_send.send((1, 3)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input input
            .output out

            in_dup(x, y) :- input(x, y).
            out(x, y) :- in_dup(x, y), in_dup(y, x).
            "#
        );

        flow.run_available();
    })
    .join()
    .unwrap();

    assert_eq!(out_recv.recv().await.unwrap(), (2, 1));
    assert_eq!(out_recv.recv().await.unwrap(), (1, 2));
    assert_eq!(out_recv.recv().await, None);
}

#[tokio::test]
pub async fn test_join_with_other() {
    let (in1_send, in1) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    in1_send.send((1, 2)).unwrap();
    in2_send.send((2, 1)).unwrap();
    in1_send.send((1, 3)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input in1
            .input in2
            .output out

            out(x, y) :- in1(x, y), in2(y, x).
            "#
        );

        flow.run_available();
    })
    .join()
    .unwrap();

    assert_eq!(out_recv.recv().await.unwrap(), (1, 2));
    assert_eq!(out_recv.recv().await, None);
}

#[tokio::test]
pub async fn test_multiple_contributors() {
    let (in1_send, in1) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    in1_send.send((1, 2)).unwrap();
    in2_send.send((3, 1)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input in1
            .input in2
            .output out

            out(x, y) :- in1(x, y).
            out(x, y) :- in2(y, x).
            "#
        );

        flow.run_available();
    })
    .join()
    .unwrap();

    assert_eq!(out_recv.recv().await.unwrap(), (1, 2));
    assert_eq!(out_recv.recv().await.unwrap(), (1, 3));
    assert_eq!(out_recv.recv().await, None);
}

#[tokio::test]
pub async fn test_transitive_closure() {
    let (edges_send, edges) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
    let (seed_reachable_send, seed_reachable) = tokio::sync::mpsc::unbounded_channel::<(usize,)>();
    let (reachable, mut reachable_recv) = tokio::sync::mpsc::unbounded_channel::<(usize,)>();

    seed_reachable_send.send((1,)).unwrap();
    edges_send.send((3, 4)).unwrap();
    edges_send.send((1, 2)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input edges
            .input seed_reachable
            .output reachable

            reachable(x) :- seed_reachable(x).
            reachable(y) :- reachable(x), edges(x, y).
            "#
        );

        flow.run_available();
    })
    .join()
    .unwrap();

    assert_eq!(reachable_recv.recv().await.unwrap(), (1,));
    assert_eq!(reachable_recv.recv().await.unwrap(), (2,));
    assert_eq!(reachable_recv.recv().await, None);
}
