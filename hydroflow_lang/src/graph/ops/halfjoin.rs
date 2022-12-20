use super::{
    parse_single_lifetime, OperatorConstraints, OperatorWriteOutput, Persistence, WriteContextArgs,
    WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;
use syn::parse_quote;

/// > 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>
///
/// Forms the "streaming half-join" of the tuples in the input streams by their first (key)
/// attribute. Similar to [`join`](#join), but only the first input stream is stored.
/// TODO:MINGWEI ELABORATE
///
/// Note that the result nests the 2nd input field (values) into a tuple in the 2nd
/// output field.
///
/// ```hydroflow
/// // should print `(hello, (world, cleveland))`
/// my_join = join();
/// source_iter(vec![("hello", "world"), ("stay", "gold")]) -> [0]my_join;
/// source_iter(vec![("hello", "cleveland")]) -> [1]my_join;
/// my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// ```
///
/// `join` can also be provided with a single generic lifetime argument, either `'epoch` or
/// `'static` to specify how join data persists. With `'epoch`, pairs will only be joined with
/// corresponding pairs within the same epoch. With `'static`, pairs will be remembered across
/// epochs and will be joined with pairs arriving in later epochs. When not explicitly specified
/// persistence defaults to `static.
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     my_join = join::<'epoch>();
///     source_iter([("hello", "world")]) -> [0]my_join;
///     source_stream(input_recv) -> [1]my_join;
///     my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_epoch();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_epoch();
/// ```
/// Prints out "(hello, (world, oakland))" since `source_iter([("hello", "world")])` is only included
/// in the first epoch, then forgotten.
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
/// flow.run_epoch();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_epoch();
/// ```
/// Prints out "(hello, (world, oakland))" and "(hello, (world, san francisco))" since the inputs
/// are peristed across epochs.
#[hydroflow_internalmacro::operator_docgen]
pub const JOIN: OperatorConstraints = OperatorConstraints {
    name: "join",
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: Some(&(|| parse_quote! { 0, 1 })),
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 wi @ &WriteIteratorArgs { ident, inputs, .. },
                 diagnostics| {
        let persistence = parse_single_lifetime(wi, diagnostics).unwrap_or(Persistence::Static);

        let joindata_ident = wc.make_ident("joindata");
        let borrow_ident = wc.make_ident("joindata_borrow");
        let (init, borrow) = match persistence {
            Persistence::Epoch => (
                quote_spanned! {op_span=>
                    #root::lang::monotonic_map::MonotonicMap::new_init(
                        #root::lang::clear::ClearDefault(
                            #root::compiled::pull::JoinState::default()
                        )
                    )
                },
                quote_spanned! {op_span=>
                    &mut #borrow_ident.try_insert_with(context.current_epoch(), Default::default).0
                },
            ),
            Persistence::Static => (
                quote_spanned! {op_span=>
                    #root::compiled::pull::JoinState::default()
                },
                quote_spanned! {op_span=>
                    &mut *#borrow_ident
                },
            ),
        };

        let write_prologue = quote_spanned! {op_span=>
            let #joindata_ident = df.add_state(std::cell::RefCell::new(
                #init
            ));
        };

        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let mut #borrow_ident = context.state_ref(#joindata_ident).borrow_mut();
            let #ident = {
                /// Limit error propagation by bounding locally, erasing output iterator type.
                #[inline(always)]
                fn check_inputs<'a, K, I1, V1, I2, V2>(
                    lhs: I1,
                    rhs: I2,
                    state: &'a mut #root::compiled::pull::JoinState<K, V1, V2>,
                ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                where
                    K: Eq + std::hash::Hash + Clone,
                    V1: Eq + Clone,
                    V2: Eq + Clone,
                    I1: 'a + Iterator<Item = (K, V1)>,
                    I2: 'a + Iterator<Item = (K, V2)>,
                {
                    #root::compiled::pull::SymmetricHashJoin::new(lhs, rhs, state)
                }
                check_inputs(#lhs, #rhs, #borrow)
            };
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
