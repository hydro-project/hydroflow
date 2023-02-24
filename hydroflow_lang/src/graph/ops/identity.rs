use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, IDENTITY_WRITE_FN, RANGE_0, RANGE_1,
};

/// > 1 input stream of type T, 1 output stream of type T
///
/// For each item passed in, pass it out without any change.
///
/// ```hydroflow
/// // should print "hello" and "world" on separate lines (in either order)
/// source_iter(vec!["hello", "world"]) -> identity()
///     -> for_each(|x| println!("{}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const IDENTITY: OperatorConstraints = OperatorConstraints {
    name: "identity",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        tainted: false,
    },
    write_fn: IDENTITY_WRITE_FN,
};
