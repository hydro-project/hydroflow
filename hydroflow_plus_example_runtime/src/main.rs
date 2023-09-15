use hydroflow_plus_example_flow;
use hydroflow_plus_example_flow::my_example_flow;

fn main() {
    let multiplier = std::env::args()
        .nth(1)
        .expect("Expected multiplier as first argument")
        .parse::<u32>()
        .expect("Expected multiplier to be a number");
    let mut flow = my_example_flow!(1, multiplier, "hi");
    flow.run_tick();
}
