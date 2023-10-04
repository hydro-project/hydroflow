use quote::quote_spanned;

use super::{
    FlowPropArgs, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{FlowProps, LatticeFlowType};

/// Stores each item as it passes through, and replays all item every tick.
///
/// ```hydroflow
/// // Normally `source_iter(...)` only emits once, but `persist()` will replay the `"hello"`
/// // on every tick.
/// source_iter(["hello"])
///     -> persist()
///     -> assert_eq(["hello"]);
/// ```
///
/// `persist()` can be used to introduce statefulness into stateless pipelines. In the example below, the
/// join only stores data for single tick. The `persist()` operator introduces statefulness
/// across ticks. This can be useful for optimization transformations within the hydroflow
/// compiler. Equivalently, we could specify that the join has `static` persistence (`my_join = join::<'static>()`).
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_iter([("hello", "world")]) -> persist() -> [0]my_join;
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
pub const PERSIST: OperatorConstraints = OperatorConstraints {
    name: "persist",
    categories: &[OperatorCategory::Persistence],
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
        monotonic: FlowPropertyVal::Yes,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    flow_prop_fn: Some(
        |fp @ FlowPropArgs {
             op_span, op_name, ..
         },
         diagnostics| {
            let input_flow_type = fp.flow_props_in[0].and_then(|fp| fp.lattice_flow_type);
            let lattice_flow_type = match input_flow_type {
                Some(LatticeFlowType::Delta) => Some(LatticeFlowType::Cumul),
                Some(LatticeFlowType::Cumul) => {
                    diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Warning,
                    format!("`{}` input is already cumulative lattice flow, this operator is redundant.", op_name),
                ));
                    Some(LatticeFlowType::Cumul)
                }
                // Non-lattice peresist.
                None => None,
            };
            Ok(vec![Some(FlowProps {
                star_ord: fp.new_star_ord(),
                lattice_flow_type,
            })])
        },
    ),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   ..
               },
               _| {
        let persistdata_ident = wc.make_ident("persistdata");
        let vec_ident = wc.make_ident("persistvec");
        let write_prologue = quote_spanned! {op_span=>
            let #persistdata_ident = #hydroflow.add_state(::std::cell::RefCell::new(
                ::std::vec::Vec::new(),
            ));
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let #ident = {
                    if context.is_first_run_this_tick() {
                        #vec_ident.extend(#input);
                        #vec_ident.iter().cloned()
                    } else {
                        let len = #vec_ident.len();
                        #vec_ident.extend(#input);
                        #vec_ident[len..].iter().cloned()
                    }
                };
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let #ident = {
                    fn constrain_types<'ctx, Push, Item>(vec: &'ctx mut Vec<Item>, mut output: Push, is_new_tick: bool) -> impl 'ctx + #root::pusherator::Pusherator<Item = Item>
                    where
                        Push: 'ctx + #root::pusherator::Pusherator<Item = Item>,
                        Item: ::std::clone::Clone,
                    {
                        if is_new_tick {
                            vec.iter().cloned().for_each(|item| {
                                #root::pusherator::Pusherator::give(&mut output, item);
                            });
                        }
                        #root::pusherator::map::Map::new(|item| {
                            vec.push(item);
                            vec.last().unwrap().clone()
                        }, output)
                    }
                    constrain_types(&mut *#vec_ident, #output, context.is_first_run_this_tick())
                };
            }
        };

        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph(), false);
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
