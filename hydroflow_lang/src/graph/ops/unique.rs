use crate::diagnostic::{Diagnostic, Level};

use super::{
    parse_persistence_lifetimes, DelayType, OperatorConstraints, OperatorWriteOutput, Persistence,
    WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;
use syn::spanned::Spanned;

/// Takes one stream as input and filters out any duplicate occurrences. The output
/// contains all unique values from the input.
///
/// ```hydroflow
/// // should print 1, 2, 3 (in any order)
/// source_iter(vec![1, 1, 2, 3, 2, 1, 3])
///     -> unique()
///     -> for_each(|x| println!("{}", x));
/// ```
///
/// `unique` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. The default is `'static`.
/// With `'tick`, uniqueness is only considered within the current tick, so across multiple ticks
/// duplicate values may be emitted.
/// With `'static`, values will be remembered across ticks and no duplicates will ever be emitted.
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> unique::<'tick>()
///         -> for_each(|n| println!("{}", n));
/// };
///
/// input_send.send(3).unwrap();
/// input_send.send(3).unwrap();
/// input_send.send(4).unwrap();
/// input_send.send(3).unwrap();
/// flow.run_available();
/// // 3, 4
///
/// input_send.send(3).unwrap();
/// input_send.send(5).unwrap();
/// flow.run_available();
/// // 3, 5
/// // Note: 3 is emitted again.
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const UNIQUE: OperatorConstraints = OperatorConstraints {
    name: "unique",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|wc @ &WriteContextArgs { op_span, .. },
                 wi @ &WriteIteratorArgs {
                     ident,
                     inputs,
                     is_pull,
                     generic_args,
                     op_name,
                     ..
                 },
                 diagnostics| {
        assert!(is_pull);

        let generics_span = generic_args.map(Spanned::span).unwrap_or(op_span);

        let persistence = parse_persistence_lifetimes(wi, diagnostics);
        let persistence = match *persistence {
            [] => Persistence::Static,
            [a] => a,
            _ => {
                diagnostics.push(Diagnostic::spanned(
                    generics_span,
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
        match persistence {
            Persistence::Tick => {
                let write_iterator = quote_spanned! {op_span=>
                    let #ident = #input.fold(
                        ::std::collections::HashSet::new(),
                        |mut prev, nxt| {prev.insert(nxt); prev}
                    ).into_iter();
                };
                Ok(OperatorWriteOutput {
                    write_iterator,
                    ..Default::default()
                })
            }
            Persistence::Static => {
                let uniquedata_ident = wc.make_ident("uniquedata");

                let write_prologue = quote_spanned! {op_span=>
                    let #uniquedata_ident = df.add_state(::std::cell::RefCell::new(::std::collections::HashSet::new()));
                };
                let write_iterator = quote_spanned! {op_span=>
                    // TODO(mingwei): Better data structure for this?
                    let #ident = {
                        let mut set = context.state_ref(#uniquedata_ident).borrow_mut();
                        set.extend(#input);
                        set.clone().into_iter()
                    };
                };

                Ok(OperatorWriteOutput {
                    write_prologue,
                    write_iterator,
                    ..Default::default()
                })
            }
        }
    }),
};
