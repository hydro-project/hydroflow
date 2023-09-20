use hydroflow_plus_example_flow::my_example_flow;

fn main() {
    let regex = std::env::args()
        .nth(1)
        .expect("Expected regex as first argument")
        .parse::<String>()
        .expect("Expected regex to be a string");
    let test_string = "test".to_string();
    let mut flow = my_example_flow!(1, regex, &test_string);
    flow.run_tick();
}
