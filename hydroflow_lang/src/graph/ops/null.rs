use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0,
};

use quote::quote_spanned;

/// > unbounded number of input streams of any type, unbounded number of output streams of any type.
///
/// As a source, generates nothing. As a sink, absorbs anything with no effect.
///
/// ```hydroflow
/// // should print `1, 2, 3, 4, 5, 6, a, b, c` across 9 lines
/// null() -> for_each(|_: ()| panic!());
/// source_iter([1,2,3]) -> map(|i| println!("{}", i)) -> null();
/// null_src = null();
/// null_sink = null();
/// null_src[0] -> for_each(|_: ()| panic!());
/// // note: use `for_each()` (or `inspect()`) instead of this:
/// source_iter([4,5,6]) -> map(|i| println!("{}", i)) -> [0]null_sink;
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const NULL: OperatorConstraints = OperatorConstraints {
    name: "null",
    hard_range_inn: &(0..=1),
    soft_range_inn: &(0..=1),
    hard_range_out: &(0..=1),
    soft_range_out: &(0..=1),
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Yes,
        monotonic: FlowPropertyVal::Yes,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            quote_spanned! {op_span=>
                (#(#inputs.for_each(std::mem::drop)),*);
                let #ident = std::iter::empty();
            }
        } else {
            quote_spanned! {op_span=>
                #[allow(clippy::let_unit_value)]
                let _ = (#(#outputs),*);
                let #ident = #root::pusherator::for_each::ForEach::new(std::mem::drop);
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
