use super::OperatorConstraints;
use crate::graph::{FlowProps, LatticeFlowType};

/// > 0 input streams, 1 output stream
///
/// > Arguments: An iterable Rust object.
///
/// The same as [`source_iter`](#source_iter) but marks the output as a `Delta` lattice flow type
/// for flow analysis. This is temporary, mainly for testing, until a more comprehensive system is
/// in place.
///
/// Takes the iterable object and delivers its elements downstream one by one.
///
/// The items should be lattice points although this is not currently enforced.
///
/// Note that all elements are emitted during the first tick.
///
/// ```hydroflow
///     use lattices::set_union::{SetUnionArray, SetUnionHashSet};
///
///     source_iter_delta([
///         SetUnionArray::new_from(["hello", "world"]),
///         SetUnionArray::new_from(["goodbye", "world"]),
///     ])
///         -> lattice_fold(SetUnionHashSet::default)
///         -> for_each(|x| println!("{:?}", x));
/// ```
pub const SOURCE_ITER_DELTA: OperatorConstraints = OperatorConstraints {
    name: "source_iter_delta",
    flow_prop_fn: Some(|fp, _diagnostics| {
        Ok(vec![Some(FlowProps {
            star_ord: fp.new_star_ord(),
            lattice_flow_type: Some(LatticeFlowType::Delta),
        })])
    }),
    ..super::source_iter::SOURCE_ITER
};
