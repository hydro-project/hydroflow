use hydroflow_plus::{q, quse, HydroflowNode, RuntimeData};

fn filter_by_regex(
    input: HydroflowNode<String>,
    regex: RuntimeData<String>,
) -> HydroflowNode<String> {
    quse!(regex::Regex);

    input.filter(q!({
        let regex = Regex::new(regex).unwrap();
        move |x| regex.is_match(x)
    }))
}

#[hydroflow_plus::flow]
pub fn my_example_flow(
    number_of_foreach: u32,
    regex: RuntimeData<String>,
    text: RuntimeData<&'static str>,
) {
    let source = HydroflowNode::source_iter(q!(vec!["abc".to_string(), "def".to_string()]));

    let mapped = filter_by_regex(source, regex);

    for _ in 0..number_of_foreach {
        mapped.for_each(q!(move |x| println!("passed regex {} {}", text, x)));
    }
}
