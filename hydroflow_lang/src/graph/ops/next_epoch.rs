use super::{DelayType, OperatorConstraints, IDENTITY_WRITE_FN, RANGE_1};

/// Delays all elements which pass through to the next epoch. In short,
/// execution of a hydroflow graph runs as a sequence of distinct "epochs".
/// Non-monotonic operators compute their output in terms of each epoch so
/// execution doesn't have to block, and it is up to the user to coordinate
/// data between epoch executions to achieve the desired result.
///
/// An epoch may be divided into multiple _strata_, see the [`next_stratum()`](#next_stratum)
/// operator.
///
/// In the example below `next_epoch()` is used alongside `difference()` to
/// ignore any items in the current epoch that already appeared in the previous
/// epoch.
/// ```rustbook
/// // Outputs 1 2 3 4 5 6 (on separate lines).
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     inp = recv_stream(input_recv) -> tee();
///     diff = difference() -> for_each(|x| println!("{}", x));
///     inp -> [pos]diff;
///     inp -> next_epoch() -> [neg]diff;
/// };
///
/// for x in [1, 2, 3, 4] {
///     input_send.send(x).unwrap();
/// }
/// flow.run_epoch();
///
/// for x in [3, 4, 5, 6] {
///     input_send.send(x).unwrap();
/// }
/// flow.run_epoch();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const NEXT_EPOCH: OperatorConstraints = OperatorConstraints {
    name: "next_epoch",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| Some(DelayType::Epoch),
    write_fn: IDENTITY_WRITE_FN,
};
