use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    IDENTITY_WRITE_FN, RANGE_0, RANGE_1,
};

/// Delays all elements which pass through to the next stratum (in the same
/// tick).
pub const NEXT_STRATUM: OperatorConstraints = OperatorConstraints {
    name: "next_stratum",
    categories: &[OperatorCategory::Control],
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
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: IDENTITY_WRITE_FN,
};
