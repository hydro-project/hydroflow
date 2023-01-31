use super::{
    OperatorConstraints, OperatorWriteOutput, Persistence, WriteContextArgs, WriteIteratorArgs,
    RANGE_0, RANGE_1,
};

use quote::quote_spanned;
use syn::parse_quote;

/// > 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>
///
/// Forms the equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.
///
/// ```hydroflow
/// // should print `(hello, (world, cleveland))`
/// my_join = join();
/// source_iter(vec![("hello", "world"), ("stay", "gold")]) -> [0]my_join;
/// source_iter(vec![("hello", "cleveland")]) -> [1]my_join;
/// my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// ```
///
/// `join` can also be provided with one or two generic lifetime persistence arguments, either
/// `'tick` or `'static`, to specify how join data persists. With `'tick`, pairs will only be
/// joined with corresponding pairs within the same tick. With `'static`, pairs will be remembered
/// across ticks and will be joined with pairs arriving in later ticks. When not explicitly
/// specified persistence defaults to `static.
///
/// When two persistence arguments are supplied the first maps to port `0` and the second maps to
/// port `1`.
/// When a single persistence argument is supplied, it is applied to both input ports.
/// When no persistence arguments are applied it defaults to `'static` for both.
///
/// The syntax is as follows:
/// ```hydroflow,ignore
/// join(); // Or
/// join::<'static>();
///
/// join::<'epoch>();
///
/// join::<'static, 'epoch>();
///
/// join::<'epoch, 'static>();
/// // etc.
/// ```
///
/// ### Examples
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     my_join = join::<'tick>();
///     source_iter([("hello", "world")]) -> [0]my_join;
///     source_stream(input_recv) -> [1]my_join;
///     my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_tick();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_tick();
/// ```
/// Prints out `"(hello, (world, oakland))"` since `source_iter([("hello", "world")])` is only
/// included in the first tick, then forgotten.
///
/// ---
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     my_join = join::<'static>();
///     source_iter([("hello", "world")]) -> [0]my_join;
///     source_stream(input_recv) -> [1]my_join;
///     my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_tick();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_tick();
/// ```
/// Prints out `"(hello, (world, oakland))"` and `"(hello, (world, san francisco))"` since the
/// inputs are peristed across ticks.
#[hydroflow_internalmacro::operator_docgen]
pub const JOIN: OperatorConstraints = OperatorConstraints {
    name: "join",
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=2),
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: Some(&(|| super::PortListSpec::Fixed(parse_quote! { 0, 1 }))),
    ports_out: None,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     persistence_args,
                     ..
                 },
                 _| {
        let persistences = match *persistence_args {
            [] => [Persistence::Static, Persistence::Static],
            [a] => [a, a],
            [a, b] => [a, b],
            _ => unreachable!(),
        };
        // TODO(mingwei): This is messy
        let items = persistences
            .zip(["lhs", "rhs"])
            .map(|(persistence, side)| {
                let joindata_ident = wc.make_ident(format!("joindata_{}", side));
                let borrow_ident = wc.make_ident(format!("joindata_{}_borrow", side));
                let (init, borrow) = match persistence {
                    Persistence::Tick => (
                        quote_spanned! {op_span=>
                            #root::lang::monotonic_map::MonotonicMap::new_init(
                                #root::lang::clear::ClearDefault(
                                    #root::compiled::pull::HalfJoinState::default()
                                )
                            )
                        },
                        quote_spanned! {op_span=>
                            &mut #borrow_ident.try_insert_with(context.current_tick(), Default::default).0
                        },
                    ),
                    Persistence::Static => (
                        quote_spanned! {op_span=>
                            #root::compiled::pull::HalfJoinState::default()
                        },
                        quote_spanned! {op_span=>
                            &mut #borrow_ident
                        },
                    ),
                };
                (joindata_ident, borrow_ident, init, borrow)
            });
        let [(lhs_joindata_ident, lhs_borrow_ident, lhs_init, lhs_borrow), (rhs_joindata_ident, rhs_borrow_ident, rhs_init, rhs_borrow)] =
            items;

        let write_prologue = quote_spanned! {op_span=>
            let #lhs_joindata_ident = df.add_state(std::cell::RefCell::new(
                #lhs_init
            ));
            let #rhs_joindata_ident = df.add_state(std::cell::RefCell::new(
                #rhs_init
            ));
        };

        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let mut #lhs_borrow_ident = context.state_ref(#lhs_joindata_ident).borrow_mut();
            let mut #rhs_borrow_ident = context.state_ref(#rhs_joindata_ident).borrow_mut();
            let #ident = {
                /// Limit error propagation by bounding locally, erasing output iterator type.
                #[inline(always)]
                fn check_inputs<'a, K, I1, V1, I2, V2>(
                    lhs: I1,
                    rhs: I2,
                    lhs_state: &'a mut #root::compiled::pull::HalfJoinState<K, V1, V2>,
                    rhs_state: &'a mut #root::compiled::pull::HalfJoinState<K, V2, V1>,
                ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                where
                    K: Eq + std::hash::Hash + Clone,
                    V1: Eq + Clone,
                    V2: Eq + Clone,
                    I1: 'a + Iterator<Item = (K, V1)>,
                    I2: 'a + Iterator<Item = (K, V2)>,
                {
                    #root::compiled::pull::SymmetricHashJoin::new_from_mut(lhs, rhs, lhs_state, rhs_state)
                }
                check_inputs(#lhs, #rhs, #lhs_borrow, #rhs_borrow)
            };
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
