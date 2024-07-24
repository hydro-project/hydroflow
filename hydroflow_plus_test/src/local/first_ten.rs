use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten<'a, D: LocalDeploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) {
    let process = flow.process(process_spec);
    let numbers = flow.source_iter(&process, q!(0..10));
    numbers.for_each(q!(|n| println!("{}", n)));
}

#[stageleft::entry]
pub fn first_ten_runtime<'a>(
    flow: FlowBuilder<'a, SingleProcessGraph>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    first_ten(&flow, &());
    flow.extract().optimize_default()
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    #[test]
    fn instantiate_first_ten() {
        let _ = super::first_ten_runtime!();
    }
}
