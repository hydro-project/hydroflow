use super::{OperatorCategory, OperatorConstraints, IDENTITY_WRITE_FN, RANGE_0, RANGE_1};
use crate::graph::GraphEdgeType;

// TODO(mingwei)
pub const STATE: OperatorConstraints = OperatorConstraints {
    name: "state",
    categories: &[OperatorCategory::Persistence],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: &(0..=1),
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    input_edgetype_fn: |_| Some(GraphEdgeType::Value),
    output_edgetype_fn: |_| GraphEdgeType::Reference,
    flow_prop_fn: None,
    write_fn: IDENTITY_WRITE_FN,
};
