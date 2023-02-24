use super::{FlowProperties, FlowPropertyVal};
use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// An operator that both filters and maps. It yields only the items for which the supplied closure returns `Some(value)`.
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
///
/// ```hydroflow
/// source_iter(vec!["1", "hello", "world", "2"]) -> filter_map(|s| s.parse().ok())
///     -> for_each(|x: usize| println!("{}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const FILTER_MAP: OperatorConstraints = OperatorConstraints {
    name: "filter_map",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::CodeBlock,
        monotonic: FlowPropertyVal::CodeBlock,
        tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs { root, op_span, .. },
               &WriteIteratorArgs {
                   ident,
                   inputs,
                   outputs,
                   arguments,
                   is_pull,
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = #input.filter_map(#arguments);
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::filter_map::FilterMap::new(#arguments, #output);
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
