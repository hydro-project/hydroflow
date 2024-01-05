use quote::quote_spanned;

use super::{
    OperatorCategory, OperatorConstraints, OperatorInstance, OperatorWriteOutput, Persistence,
    WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{GraphEdgeType, OpInstGenerics};

/// > 1 input stream of type `T`, 1 output stream of type `(usize, T)`
///
/// For each item passed in, enumerate it with its index: `(0, x_0)`, `(1, x_1)`, etc.
///
/// `enumerate` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify if indexing resets. If `'tick` (the default) is specified, indexing will
/// restart at zero at the start of each tick. Otherwise `'static` will never reset
/// and count monotonically upwards.
///
/// ```hydroflow
/// source_iter(vec!["hello", "world"])
///     -> enumerate()
///     -> assert_eq([(0, "hello"), (1, "world")]);
/// ```
pub const ENUMERATE: OperatorConstraints = OperatorConstraints {
    name: "enumerate",
    categories: &[OperatorCategory::Map],
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
    input_delaytype_fn: |_| None,
    output_edgetype_fn: |_| GraphEdgeType::Value,
    flow_prop_fn: None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   op_span,
                   context,
                   hydroflow,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
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
        let persistence = match persistence_args[..] {
            [] => Persistence::Tick,
            [a] => a,
            _ => unreachable!(),
        };

        let input = &inputs[0];
        let output = &outputs[0];

        let counter_ident = wc.make_ident("counterdata");

        let (write_prologue, get_counter) = match persistence {
            Persistence::Tick => (
                quote_spanned! {op_span=>
                    let #counter_ident = #hydroflow.add_state(::std::cell::RefCell::new(
                        #root::util::monotonic_map::MonotonicMap::new_init(0..),
                    ));
                },
                quote_spanned! {op_span=>
                    let mut borrow = #context.state_ref(#counter_ident).borrow_mut();
                    let counter = borrow.get_mut_with((#context.current_tick(), #context.current_stratum()), || 0..);
                },
            ),
            Persistence::Static => (
                quote_spanned! {op_span=>
                    let #counter_ident = #hydroflow.add_state(::std::cell::RefCell::new(0..));
                },
                quote_spanned! {op_span=>
                    let mut counter = #context.state_ref(#counter_ident).borrow_mut();
                },
            ),
            Persistence::Mutable => {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    "An implementation of 'mutable does not exist",
                ));
                return Err(());
            }
        };

        let map_fn = quote_spanned! {op_span=>
            |item| {
                #get_counter
                (counter.next().unwrap(), item)
            }
        };
        let write_iterator = if is_pull {
            quote_spanned! {op_span=>
                let #ident = ::std::iter::Iterator::map(#input, #map_fn);
            }
        } else {
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::map::Map::new(#map_fn, #output);
            }
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
