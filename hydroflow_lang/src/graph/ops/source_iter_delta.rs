use super::OperatorConstraints;
use crate::graph::{FlowProps, LatticeFlowType};

/// > 0 input streams, 1 output stream
///
/// > Arguments: An iterable Rust object.
/// Takes the iterable object and delivers its elements downstream
/// one by one.
///
/// Note that all elements are emitted during the first tick.
///
/// ```hydroflow
///     source_iter(vec!["Hello", "World"])
///         -> for_each(|x| println!("{}", x));
/// ```
pub const SOURCE_ITER_DELTA: OperatorConstraints = OperatorConstraints {
    name: "source_iter_delta",
    flow_prop_fn: Some(|_flow_props_in, _op_inst, star_ord| {
        vec![FlowProps {
            star_ord,
            lattice_flow_type: Some(LatticeFlowType::Delta),
        }]
    }),
    ..super::source_iter::SOURCE_ITER
};
