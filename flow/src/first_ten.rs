use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten<'a, D: LocalDeploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) {
    let process = flow.process(process_spec);

    let numbers = process.source_iter(q!(0..10));
    numbers.for_each(q!(|n| println!("{}", n)));
}

#[stageleft::entry]
pub fn first_ten_runtime<'a>(
    flow: &'a FlowBuilder<'a, SingleProcessGraph>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    first_ten(flow, &());
    flow.build_single()
}
