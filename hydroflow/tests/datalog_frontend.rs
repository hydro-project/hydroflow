use datalog_compiler::datalog;

// #[test]
// pub fn test_minimal() {
//     let (in_send, in_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

//     let mut flow = datalog!(
//         r#"
//         .input in
//         .output out

//         out(y, x) :- in(x, y).
//         "#
//     );

//     in_send.send((1, 2)).unwrap();
//     flow.run_available();
// }

#[test]
pub fn test_join_with_self() {
    let (in_send, in_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input in
        .output out

        out(x, y) :- in(x, y), in(y, x).
        "#
    );

    in_send.send((1, 2)).unwrap();
    flow.run_available();
}

// #[test]
// pub fn test_join_with_other() {
//     let (in1_send, in1_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
//     let (in2_send, in2_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

//     let mut flow = datalog!(
//         r#"
//         .input in1
//         .input in2
//         .output out

//         out(x, y) :- in1(x, y), in2(y, x).
//         "#
//     );

//     in1_send.send((1, 2)).unwrap();
//     in2_send.send((2, 1)).unwrap();
//     flow.run_available();
// }
