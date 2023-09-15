use hydroflow_plus::{q, HydroflowNode, RuntimeData};

fn multiply_numbers(
    input: HydroflowNode<u32>,
    multiplier: RuntimeData<u32>
) -> HydroflowNode<u32> {
    input.map(q!(
        move |v| v * multiplier
    ))
}

#[hydroflow_plus::flow]
pub fn my_example_flow(
    number_of_foreach: u32,
    multiplier: RuntimeData<u32>,
    text: RuntimeData<&str>,
) {
    let source = HydroflowNode::source_iter(
        q!(vec![1, 2, 3, number_of_foreach])
    );

    let mapped = multiply_numbers(
        source,
        multiplier
    );

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("{} {}", text, x)));
    }
}
