use hydroflow_plus::{q, HydroflowNode, RuntimeData};

#[hydroflow_plus::flow]
pub fn my_example_flow(
    number_of_foreach: u32,
    multiplier: RuntimeData<u32>,
    text: RuntimeData<&str>,
) {
    let source = HydroflowNode::source_iter(q!(vec![1, 2, 3, number_of_foreach]));
    let mapped = source.map(q!(move |x| x * multiplier));

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("{} {}", text, x)));
    }
}
