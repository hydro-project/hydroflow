use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorConstraints, IDENTITY_WRITE_FN, RANGE_0,
    RANGE_1,
};

/// Delays all elements which pass through to the next stratum (in the same
/// tick).
#[hydroflow_internalmacro::operator_docgen]
pub const NEXT_STRATUM: OperatorConstraints = OperatorConstraints {
    name: "next_stratum",
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
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        tainted: false,
    },
    write_fn: IDENTITY_WRITE_FN,
};
