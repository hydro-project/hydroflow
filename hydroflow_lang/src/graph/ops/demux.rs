use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Level};
use crate::graph::PortIndexValue;
use crate::pretty_span::PrettySpan;

use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use proc_macro2::{Ident, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, Pat};

// TODO(mingwei): Preprocess rustdoc links in mdbook or in the `operator_docgen` macro.
/// > Arguments: A Rust closure, the first argument is a received item and the
/// > second argument is a [`var_args!` tuple list](https://hydro-project.github.io/hydroflow/doc/hydroflow/macro.var_args.html)
/// > where each item name is an output port.
///
/// Takes the input stream and allows the user to determine what elemnt(s) to
/// deliver to any number of output streams.
///
/// > Note: Downstream operators may need explicit type annotations.
///
/// > Note: The [`Pusherator`](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html)
/// > trait is automatically imported to enable the [`.give(...)` method](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html#tymethod.give).
///
/// ```hydroflow
/// my_demux = source_iter(1..=100) -> demux(|v, var_args!(fzbz, fizz, buzz, vals)|
///     match (v % 3, v % 5) {
///         (0, 0) => fzbz.give(v),
///         (0, _) => fizz.give(v),
///         (_, 0) => buzz.give(v),
///         (_, _) => vals.give(v),
///     }
/// );
/// my_demux[fzbz] -> for_each(|v| println!("{}: fizzbuzz", v));
/// my_demux[fizz] -> for_each(|v| println!("{}: fizz", v));
/// my_demux[buzz] -> for_each(|v| println!("{}: buzz", v));
/// my_demux[vals] -> for_each(|v| println!("{}", v));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const DEMUX: OperatorConstraints = OperatorConstraints {
    name: "demux",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(2..),
    soft_range_out: &(2..),
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|&WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     outputs,
                     output_ports,
                     arguments,
                     op_name,
                     is_pull,
                     ..
                 },
                 diagnostics| {
        assert!(!is_pull);
        let func = &arguments[0];
        let Expr::Closure(func) = func else {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                "Second argument must be a two-argument closure expression"),
            );
            return Err(());
        };
        if 2 != func.inputs.len() {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                &*format!(
                    "Closure provided as second argument to `{}` must have a second \
                    input argument listing ports, `var_args!(port_a, port_b, ...)`.",
                    op_name
                ),
            ));
            return Err(());
        }

        // Port idents specified in the closure's second argument.
        let arg2 = &func.inputs[1];
        let closure_idents = extract_closure_idents(arg2);

        // Port idents supplied via port connections in the surface syntax.
        let port_idents: Vec<_> = output_ports
            .iter()
            .filter_map(|&output_port| {
                let PortIndexValue::Path(port_expr) = output_port else {
                    diagnostics.push(Diagnostic::spanned(
                        output_port.span(),
                        Level::Error,
                        format!(
                            "Output port from `{}(..)` must be specified and must be a valid identifier.",
                            op_name,
                        ),
                    ));
                    return None;
                };
                let port_ident = syn::parse2::<Ident>(quote! { #port_expr })
                    .map_err(|err| diagnostics.push(err.into()))
                    .ok()?;

                if !closure_idents.contains_key(&port_ident) {
                    // TODO(mingwei): Use MultiSpan when `proc_macro2` supports it.
                    diagnostics.push(Diagnostic::spanned(
                        arg2.span(),
                        Level::Error,
                        format!(
                            "Argument specifying the output ports in `{0}(..)` does not contain extra port `{1}`: ({2}) (1/2).",
                            op_name, port_ident, PrettySpan(output_port.span()),
                        ),
                    ));
                    diagnostics.push(Diagnostic::spanned(
                        output_port.span(),
                        Level::Error,
                        format!(
                            "Port `{1}` not found in the arguments specified in `{0}(..)`'s closure: ({2}) (2/2).",
                            op_name, port_ident, PrettySpan(arg2.span()),
                        ),
                    ));
                    return None;
                }

                Some(port_ident)
            })
            .collect();

        for closure_ident in closure_idents.keys() {
            if !port_idents.contains(closure_ident) {
                diagnostics.push(Diagnostic::spanned(
                    closure_ident.span(),
                    Level::Error,
                    format!(
                        "`{}(..)` closure argument `{}` missing corresponding output port.",
                        op_name, closure_ident,
                    ),
                ));
            }
        }

        if diagnostics.iter().any(Diagnostic::is_error) {
            return Err(());
        }

        assert_eq!(outputs.len(), port_idents.len());
        assert_eq!(outputs.len(), closure_idents.len());

        let mut sort_permute: Vec<_> = (0..outputs.len()).collect();
        sort_permute.sort_by_key(|&i| closure_idents[&port_idents[i]]);

        let sorted_outputs = sort_permute.iter().map(|&i| &outputs[i]);

        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                #[allow(unused_imports)] use #root::pusherator::Pusherator;
                #root::pusherator::demux::Demux::new(#func, #root::var_expr!( #( #sorted_outputs ),* ))
            };
        };

        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    }),
};

fn extract_closure_idents(arg2: &Pat) -> HashMap<Ident, usize> {
    let tokens = if let Pat::Macro(pat_macro) = arg2 {
        pat_macro.mac.tokens.clone()
    } else {
        arg2.to_token_stream()
    };

    let mut idents = HashMap::new();
    let mut stack: Vec<_> = tokens.into_iter().collect();
    stack.reverse();
    while let Some(tt) = stack.pop() {
        match tt {
            TokenTree::Group(group) => {
                let a = stack.len();
                stack.extend(group.stream().into_iter());
                let b = stack.len();
                stack[a..b].reverse();
            }
            TokenTree::Ident(ident) => {
                idents.insert(ident, idents.len());
            }
            TokenTree::Punct(_) => (),
            TokenTree::Literal(_) => (),
        }
    }
    idents
}
