use hydroflow::hydroflow_syntax;

fn main() {
    let (input_send, input_recv) = hydroflow::util::unbounded_channel::<&str>();
    let mut flow = hydroflow_syntax! {
        source_stream(input_recv) -> map(|x| x.to_uppercase())
            -> for_each(|x| println!("{}", x));
    };
    input_send.send("Hello").unwrap();
    input_send.send("World").unwrap();
    flow.run_available();
}
