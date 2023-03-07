use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::{quote_spanned, ToTokens};
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
/// join::<'tick>();
///
/// join::<'static, 'tick>();
///
/// join::<'tick, 'static>();
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
pub const STAMP: OperatorConstraints = OperatorConstraints {
    name: "stamp",
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=0),
    type_args: &(1..=1),
    is_external_input: false,
    ports_inn: Some(&(|| super::PortListSpec::Fixed(parse_quote! { 0, 1 }))),
    ports_out: None,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs {
                     context, op_span, ..
                 },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     type_args,
                     ..
                 },
                 _| {
        let stampdata_ident = wc.make_ident("stamp_data");

        let stamp_type = type_args
            .get(0)
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let write_prologue = quote_spanned! {op_span=>

            let #stampdata_ident = df.add_state(std::cell::RefCell::new(
                #stamp_type::default()
            ));
        };

        let stamp_borrow = wc.make_ident("stamp_borrow");
        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let mut #stamp_borrow = #context.state_ref(#stampdata_ident).borrow_mut();

            let #ident = {
                /// Limit error propagation by bounding locally, erasing output iterator type.
                #[inline(always)]
                fn check_inputs<'a, I1, V1, I2, V2>(
                    mut lhs: I1,
                    rhs: I2,
                    stampdata: &'a mut V1,
                ) -> impl 'a + Iterator<Item = (V1, V2)>
                where
                    V1: Eq + Clone + CvRDT + Default,
                    V2: Eq + Clone,
                    I1: 'a + Iterator<Item = V1>,
                    I2: 'a + Iterator<Item = V2>,
                {
                    while let Some(s) = lhs.next() {
                        stampdata.merge(s);
                    }

                    rhs.map(|x| (stampdata.clone(), x))
                }
                check_inputs(#lhs, #rhs, &mut *#stamp_borrow)
            };
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
