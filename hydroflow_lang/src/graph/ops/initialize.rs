use syn::parse_quote_spanned;

use crate::graph::OperatorInstance;

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, WriteContextArgs, RANGE_0, RANGE_1,
};

/// > 0 input streams, 1 output stream
///
/// > Arguments: None.
///
/// Emits a single unit `()` at the start of the first tick.
///
/// ```hydroflow
/// initialize() -> for_each(|()| println!("This only runs one time!"));
/// ```
pub const INITIALIZE: OperatorConstraints = OperatorConstraints {
    name: "initialize",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
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
    write_fn: |wc @ &WriteContextArgs { op_span, .. }, diagnostics| {
        let wc = WriteContextArgs {
            op_inst: &OperatorInstance {
                arguments: parse_quote_spanned!(op_span=> [()]),
                ..wc.op_inst.clone()
            },
            ..wc.clone()
        };
        (super::source_iter::SOURCE_ITER.write_fn)(&wc, diagnostics)
    },
};
