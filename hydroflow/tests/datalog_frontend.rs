use datalog_compiler::datalog;

#[test]
pub fn test_simple() {
    let (in_send, in_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
    .input in
    .output out

    out(x, y) :- in(x, y).
  "#
    );

    in_send.send((1, 2)).unwrap();
    flow.run_available();
}
