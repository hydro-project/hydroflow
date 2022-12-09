use super::{OperatorConstraints, IDENTITY_WRITE_FN, RANGE_1};

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
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| None,
    write_fn: IDENTITY_WRITE_FN,
};
