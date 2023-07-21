use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    IDENTITY_WRITE_FN, RANGE_0, RANGE_1,
};

/// Buffers all input items and releases them in the next tick.
/// the state of the current tick. For example,
/// See the [book discussion of Hydroflow time](../concepts/life_and_times) for details on ticks.
/// A tick may be divided into multiple [strata](../concepts/stratification); see the [`next_stratum()`](#next_stratum)
/// operator.
///
/// `defer_tick` is sometimes needed to separate conflicting data across time,
/// in order to preserve invariants. Consider the following example. The `Point` lattice
/// has an immutability invariant: if you merge a new value into it, that is a runtime error.
/// Here we ensure that the `lattice_reduce` in `state` will have exactly one value per tick,
/// and that value will be the negation of the previous value.
/// ```rustbook
/// // Outputs Point { val: true, _token: PhantomData<*mut ()> }, Point { val: false, _token: PhantomData<*mut ()> }, ...
/// use hydroflow::hydroflow_syntax;
/// use lattices::Point;
///
/// pub fn main() {
///     let mut df = hydroflow_syntax! {
///             source_iter(vec!(Point::<bool, ()>::new(true)))
///                     -> state;
///             state = union()
///                     -> lattice_reduce::<'tick, Point<bool, ()>>()
///                     -> inspect(|x| println!("{:?}", x));
///             state -> map(|x| Point::<_, ()>::new(!x.as_reveal_ref()))
///                   -> defer_tick()
///                   -> state;
///     };
///     df.run_available();
/// }

/// pub fn main() {
//     let mut df = hydroflow_syntax! {
//             source_iter(vec!(Point::<bool, ()>::new(true)))
//                     -> state;
//             state = union()
//                     -> lattice_reduce::<'tick, Point<bool, ()>>()
//                     -> inspect(|x| println!("{:?}", x))
//                     -> map(|x| Point::<_, ()>::new(!x.as_reveal_ref()))
//                     -> defer_tick()
//                     -> state;
//     };
//     df.run_available();
// }
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
/// 
/// `defer_tick` can also be handy for comparing stream content across ticks.
/// In the example below `defer_tick()` is used alongside `difference()` to
/// filter out any items that arrive from `inp` in the current tick which match
/// an item from `inp` in the previous
/// tick.
/// ```rustbook
/// // Outputs 1 2 3 4 5 6 (on separate lines).
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     inp = source_stream(input_recv) -> tee();
///     inp -> [pos]diff;
///     inp -> defer_tick() -> [neg]diff;
///     diff = difference() -> for_each(|x| println!("{}", x));
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
pub const DEFER_TICK: OperatorConstraints = OperatorConstraints {
    name: "defer_tick",
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
    input_delaytype_fn: |_| Some(DelayType::Tick),
    write_fn: IDENTITY_WRITE_FN,
};
