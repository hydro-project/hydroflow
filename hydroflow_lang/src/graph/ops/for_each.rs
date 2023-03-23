use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorInstance, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 0 output streams
///
/// > Arguments: a Rust closure
///
/// Iterates through a stream passing each element to the closure in the
/// argument.
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
///
/// ```hydroflow
///     source_iter(vec!["Hello", "World"])
///         -> for_each(|x| println!("{}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const FOR_EACH: OperatorConstraints = OperatorConstraints {
    name: "for_each",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_0,
    soft_range_out: RANGE_0,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::DependsOnArgs,
        monotonic: FlowPropertyVal::DependsOnArgs,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let write_iterator = quote_spanned! {op_span=>
            let #ident = #root::pusherator::for_each::ForEach::new(#arguments);
        };
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    },
};
