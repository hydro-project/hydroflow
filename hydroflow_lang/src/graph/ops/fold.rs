use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput,
    Persistence, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// > Arguments: an initial value, and a closure which itself takes two arguments:
/// an 'accumulator', and an element. The closure returns the value that the accumulator should have for the next iteration.
///
/// Akin to Rust's built-in fold operator. Folds every element into an accumulator by applying a closure,
/// returning the final result.
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
///
/// `fold` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `'static`.
///
/// ```hydroflow
/// // should print `Reassembled vector [1,2,3,4,5]`
/// source_iter([1,2,3,4,5])
///     -> fold::<'tick>(Vec::new(), |mut accum, elem| {
///         accum.push(elem);
///         accum
///     })
///     -> for_each(|e| println!("Ressembled vector {:?}", e));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const FOLD: OperatorConstraints = OperatorConstraints {
    name: "fold",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 2,
    persistence_args: &(0..=1),
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::CodeBlock,
        monotonic: FlowPropertyVal::CodeBlock,
        tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |wc @ &WriteContextArgs {
                   context, op_span, ..
               },
               &WriteIteratorArgs {
                   ident,
                   inputs,
                   persistence_args,
                   arguments,
                   is_pull,
                   ..
               },
               _| {
        assert!(is_pull);

        let persistence = match *persistence_args {
            [] => Persistence::Static,
            [a] => a,
            _ => unreachable!(),
        };

        let input = &inputs[0];
        let init = &arguments[0];
        let func = &arguments[1];
        let folddata_ident = wc.make_ident("folddata");

        let (write_prologue, write_iterator, write_iterator_after) = match persistence {
            // TODO(mingwei): Issues if initial value is not copy.
            // TODO(mingwei): Might introduce the initial value multiple times on scheduling.
            Persistence::Tick => (
                Default::default(),
                quote_spanned! {op_span=>
                    #[allow(clippy::unnecessary_fold)]
                    let #ident = ::std::iter::once(#input.fold(#init, #func));
                },
                Default::default(),
            ),
            Persistence::Static => (
                quote_spanned! {op_span=>
                    let #folddata_ident = df.add_state(
                        ::std::cell::Cell::new(::std::option::Option::Some(#init))
                    );
                },
                quote_spanned! {op_span=>
                    let #ident = {
                        let accum = #context.state_ref(#folddata_ident).take().expect("FOLD DATA MISSING");
                        #[allow(clippy::unnecessary_fold)]
                        let accum = #input.fold(accum, #func);
                        #context.state_ref(#folddata_ident).set(
                            ::std::option::Option::Some(::std::clone::Clone::clone(&accum))
                        );
                        ::std::iter::once(accum)
                    };
                },
                quote_spanned! {op_span=>
                    #context.schedule_subgraph(#context.current_subgraph());
                },
            ),
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
