use super::{
    parse_persistence_lifetimes, DelayType, OperatorConstraints, OperatorWriteOutput, Persistence,
    WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;

use crate::diagnostic::{Diagnostic, Level};

/// > 1 input stream of type (K,V1), 1 output stream of type (K,V2).
/// The output will have one tuple for each distinct K, with an accumulated value of type V2.
///
/// > Arguments: two Rust closures. The first generates an initial value per group. The second
/// itself takes two arguments: an 'accumulator', and an element. The second closure returns the
/// value that the accumulator should have for the next iteration.
///
/// A special case of `fold`, in the spirit of SQL's GROUP BY and aggregation constructs. The input
/// is partitioned into groups by the first field, and for each group the values in the second
/// field are accumulated via the closures in the arguments.
///
/// `group_by` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `static.
///
/// ```hydroflow
/// source_iter([("toy", 1), ("toy", 2), ("shoe", 11), ("shoe", 35), ("haberdashery", 7)])
///     -> group_by(|| 0, |old: &mut u32, val: u32| *old += val)
///     -> for_each(|(k, v)| println!("Total for group {} is {}", k, v));
/// ```
///
/// Example using `'tick` persistence:
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> group_by::<'tick>(|| String::new(), |old: &mut String, val: &str| {
///             *old += val;
///             *old += ", ";
///         })
///         -> for_each(|(k, v)| println!("({:?}, {:?})", k, v));
/// };
///
/// input_send.send(("hello", "oakland")).unwrap();
/// input_send.send(("hello", "berkeley")).unwrap();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_available();
/// // ("hello", "oakland, berkeley, san francisco, ")
///
/// input_send.send(("hello", "palo alto")).unwrap();
/// flow.run_available();
/// // ("hello", "palo alto, ")
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const GROUP_BY: OperatorConstraints = OperatorConstraints {
    name: "group_by",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 2,
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|wc @ &WriteContextArgs { op_span, .. },
                 wi @ &WriteIteratorArgs {
                     ident,
                     inputs,
                     arguments,
                     is_pull,
                     op_name,
                     ..
                 },
                 diagnostics| {
        assert!(is_pull);

        let persistence = parse_persistence_lifetimes(wi, diagnostics);
        let persistence = match *persistence {
            [] => Persistence::Static,
            [a] => a,
            _ => {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    format!(
                        "Operator `{}` expects zero or one persistence lifetime generic arguments",
                        op_name
                    ),
                ));
                Persistence::Static
            }
        };

        let input = &inputs[0];
        let initfn = &arguments[0];
        let aggfn = &arguments[1];
        let groupbydata_ident = wc.make_ident("groupbydata");

        let (write_prologue, write_iterator) = match persistence {
            Persistence::Tick => (
                Default::default(),
                quote_spanned! {op_span=>
                    let #ident = {
                        #[inline(always)]
                        fn check_input<Iter: ::std::iter::Iterator<Item = (A, B)>, A, B>(iter: Iter) -> impl ::std::iter::Iterator<Item = (A, B)> { iter }
                        check_input(#input).fold(::std::collections::HashMap::new(), |mut ht, (key, val)| {
                            let entry = ht.entry(key).or_insert_with(#initfn);
                            #[allow(clippy::redundant_closure_call)] (#aggfn)(entry, val);
                            ht
                        }).into_iter()
                    };
                },
            ),
            Persistence::Static => (
                quote_spanned! {op_span=>
                    let #groupbydata_ident = df.add_state(::std::cell::RefCell::new(::std::collections::HashMap::new()));
                },
                quote_spanned! {op_span=>
                    let #ident = {
                        let mut ht = context.state_ref(#groupbydata_ident).borrow_mut();
                        #[inline(always)]
                        fn check_input<Iter: ::std::iter::Iterator<Item = (A, B)>, A: ::std::clone::Clone, B: ::std::clone::Clone>(iter: Iter)
                            -> impl ::std::iter::Iterator<Item = (A, B)> { iter }
                        let mut any = false;
                        for (key, val) in check_input(#input) {
                            any = true;
                            let entry = ht.entry(key).or_insert_with(#initfn);
                            #[allow(clippy::redundant_closure_call)] (#aggfn)(entry, val);
                        }
                        ::std::iter::IntoIterator::into_iter(
                            if any {
                                // TODO(mingwei): extra collect here, could be avoided by keeping the `BorrowMut` alive (risky?).
                                ht.iter()
                                    .map(#[allow(clippy::clone_on_copy)] |(k, v)| (k.clone(), v.clone()))
                                    .collect::<::std::vec::Vec::<_>>()
                            }
                            else {
                                ::std::vec::Vec::new()
                            }
                        )
                    };
                },
            ),
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
