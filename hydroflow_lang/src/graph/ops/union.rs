use quote::{quote_spanned, ToTokens};

use super::{
    FlowPropArgs, FlowProps, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1, RANGE_ANY,
};
use crate::diagnostic::{Diagnostic, Level};

/// > *n* input streams of the same type, 1 output stream of the same type
///
/// Unions an arbitrary number of input streams into a single stream. Each input sequence is a subsequence of the output, but no guarantee is given on how the inputs are interleaved.
///
/// Since `union` has multiple input streams, it needs to be assigned to
/// a variable to reference its multiple input ports across statements.
///
/// ```hydroflow
/// source_iter(vec!["hello", "world"]) -> my_union;
/// source_iter(vec!["stay", "gold"]) -> my_union;
/// source_iter(vec!["don't", "give", "up"]) -> my_union;
/// my_union = union()
///     -> map(|x| x.to_uppercase())
///     -> assert_eq(["HELLO", "WORLD", "STAY", "GOLD", "DON'T", "GIVE", "UP"]);
/// ```
pub const UNION: OperatorConstraints = OperatorConstraints {
    name: "union",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: RANGE_ANY,
    soft_range_inn: &(2..),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    has_singleton_output: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    flow_prop_fn: Some(
        |fp @ FlowPropArgs {
             op_name,
             op_inst,
             flow_props_in,
             ..
         },
         diagnostics| {
            let lattice_flow_type = flow_props_in
                .iter()
                .map(|flow_props| flow_props.and_then(|fp| fp.lattice_flow_type))
                .reduce(std::cmp::min)
                .flatten();
            // Warn if we are doing any implied downcasting.
            for (fp_in, port_in) in flow_props_in.iter().zip(op_inst.input_ports.iter()) {
                if lattice_flow_type != fp_in.and_then(|fp| fp.lattice_flow_type) {
                    diagnostics.push(Diagnostic::spanned(
                        port_in.span(),
                        Level::Warning,
                        format!(
                            "Input to `{}()` will be downcast to `{:?}` to match other inputs.",
                            op_name, lattice_flow_type,
                        ),
                    ));
                }
            }
            Ok(vec![Some(FlowProps {
                star_ord: fp.new_star_ord(),
                lattice_flow_type,
            })])
        },
    ),
    write_fn: |&WriteContextArgs {
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            let chains = inputs
                .iter()
                .map(|i| i.to_token_stream())
                .reduce(|a, b| quote_spanned! {op_span=> check_inputs(#a, #b) })
                .unwrap_or_else(|| quote_spanned! {op_span=> std::iter::empty() });
            quote_spanned! {op_span=>
                let #ident = {
                    #[allow(unused)]
                    #[inline(always)]
                    fn check_inputs<A: ::std::iter::Iterator<Item = Item>, B: ::std::iter::Iterator<Item = Item>, Item>(a: A, b: B) -> impl ::std::iter::Iterator<Item = Item> {
                        a.chain(b)
                    }
                    #chains
                };
            }
        } else {
            assert_eq!(1, outputs.len());
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #output;
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
