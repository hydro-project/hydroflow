#![allow(clippy::needless_lifetimes)]

use hydroflow_plus::{*, deploy::SingleProcessGraph};
use stageleft::*;

pub fn first_ten<'a>(
    flow: &FlowBuilder<'a>,
) {
    let process = flow.process();

    let numbers = flow.source_iter(&process, q!(0..10)); // : Stream<_, i32, _, _>
    numbers.for_each(q!(|n| println!("{}", n)));
}

#[stageleft::entry]
pub fn first_ten_runtime<'a>(
    flow: FlowBuilder<'a>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    first_ten(&flow);
    flow.with_default_optimize().compile_no_network::<SingleProcessGraph>() // : impl Quoted<'a, Hydroflow<'a>>
}
