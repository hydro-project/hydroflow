use super::{DelayType, OperatorConstraints, IDENTITY_WRITE_FN, RANGE_1};

/// Delays all elements which pass through to the next tick. In short,
/// execution of a hydroflow graph runs as a sequence of distinct "ticks".
/// Non-monotonic operators compute their output in terms of each tick so
/// execution doesn't have to block, and it is up to the user to coordinate
/// data between tick executions to achieve the desired result.
///
/// An tick may be divided into multiple _strata_, see the [`next_stratum()`](#next_stratum)
/// operator.
///
/// In the example below `next_tick()` is used alongside `difference()` to
/// ignore any items in the current tick that already appeared in the previous
/// tick.
/// ```rustbook
/// // Outputs 1 2 3 4 5 6 (on separate lines).
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     inp = source_stream(input_recv) -> tee();
///     diff = difference() -> for_each(|x| println!("{}", x));
///     inp -> [pos]diff;
///     inp -> next_tick() -> [neg]diff;
/// };
///
/// for x in [1, 2, 3, 4] {
///     input_send.send(x).unwrap();
/// }
/// flow.run_tick();
///
/// for x in [3, 4, 5, 6] {
///     input_send.send(x).unwrap();
/// }
/// flow.run_tick();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const NEXT_TICK: OperatorConstraints = OperatorConstraints {
    name: "next_tick",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: &|_| Some(DelayType::Tick),
    write_fn: IDENTITY_WRITE_FN,
};
