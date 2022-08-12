use datalog_compiler::datalog;

#[test]
pub fn test_simple() {
  let flow = datalog!(r#"
    .input edge
    .output path

    path(x, y) :- edge(x, y)
    path(x, y) :- path(x, z), edge(z, y).
  "#);
}