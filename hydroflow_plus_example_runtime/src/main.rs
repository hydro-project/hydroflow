use hydroflow_plus_example_flow::my_example_flow;

fn main() {
    let (send, recv) = hydroflow_plus::hydroflow::util::unbounded_channel::<String>();
    let regex = std::env::args()
        .nth(1)
        .expect("Expected regex as first argument")
        .parse::<String>()
        .expect("Expected regex to be a string");
    let test_string = "test".to_string();
    let mut flow = my_example_flow!(recv, 1, &regex, &test_string);
    send.send("abc".to_string()).unwrap();
    send.send("def".to_string()).unwrap();
    flow.run_tick();
}
