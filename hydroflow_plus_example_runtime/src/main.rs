use hydroflow_plus_example_flow::my_example_flow;

fn main() {
    let mut flow = my_example_flow!(1, 123, "hi");
    flow.run_tick();
}
