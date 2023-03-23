use crate::{
    diagnostic::{Diagnostic, Level},
    graph::{OpInstGenerics, OperatorInstance},
};

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, Persistence,
    WriteContextArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// Stores each item as it passes through, and replays all item every tick.
///
/// ```hydroflow
/// // Normally `source_iter(...)` only emits once, but with `persist()` will replay the `"hello"`
/// // on every tick. This is equivalent to `repeat_iter(["hello"])`.
/// source_iter(["hello"])
///     -> persist()
///     -> for_each(|item| println!("{}: {}", context.current_tick(), item));
/// ```
///
/// `persist()` can be used to introduce statefulness into stateless pipelines. For example this
/// join only stores data for single `'tick`. The `persist()` operator introduces statefulness
/// across ticks. This can be useful for optimization transformations within the hydroflow
/// compiler.
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     repeat_iter([("hello", "world")]) -> [0]my_join;
///     source_stream(input_recv) -> persist() -> [1]my_join;
///     my_join = join::<'tick>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_tick();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_tick();
/// // (hello, (world, oakland))
/// // (hello, (world, oakland))
/// // (hello, (world, san francisco))
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const PERSIST: OperatorConstraints = OperatorConstraints {
    name: "persist",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=1),
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Yes,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   op_name,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   persistence_args, ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {
        if matches!(persistence_args[..], [Persistence::Tick]) {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                format!("`{}()` can only have `'static` persistence.", op_name),
            ));
        };

        let persistdata_ident = wc.make_ident("persistdata");
        let vec_ident = wc.make_ident("persistvec");
        let write_prologue = quote_spanned! {op_span=>
            let #persistdata_ident = #hydroflow.add_state(::std::cell::RefCell::new(::std::vec::Vec::new()));
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                #vec_ident.extend(#input);
                let #ident = #vec_ident.iter().cloned();
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let #ident = {
                    fn constrain_types<'ctx, Push, Item>(vec: &'ctx mut Vec<Item>, mut output: Push) -> impl 'ctx + #root::pusherator::Pusherator<Item = Item>
                    where
                        Push: 'ctx + #root::pusherator::Pusherator<Item = Item>,
                        Item: ::std::clone::Clone,
                    {
                        vec.iter().cloned().for_each(|item| {
                            #root::pusherator::Pusherator::give(&mut output, item);
                        });
                        #root::pusherator::map::Map::new(|item| {
                            vec.push(item);
                            vec.last().unwrap().clone()
                        }, output)
                    }
                    constrain_types(&mut *#vec_ident, #output)
                };
            }
        };

        OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        }
    },
};
