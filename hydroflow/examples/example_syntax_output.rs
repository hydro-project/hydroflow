use hydroflow::hydroflow_syntax;

fn main() {
    let (output_send, mut output_recv) = hydroflow::util::unbounded_channel::<char>();
    let mut flow = hydroflow_syntax! {
        source_iter("Hello World".chars()) -> map(|c| c.to_ascii_uppercase())
            -> for_each(|c| output_send.send(c).unwrap());
    };
    flow.run_available();

    let output = &*hydroflow::util::collect_ready::<String, _>(&mut output_recv);
    assert_eq!(output, "HELLO WORLD");
}
