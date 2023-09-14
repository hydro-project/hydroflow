use std::collections::HashMap;

use proc_macro2::{Ident, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, Pat};

use super::{
    FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    PortListSpec, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{OpInstGenerics, OperatorInstance, PortIndexValue};
use crate::pretty_span::PrettySpan;

// TODO(mingwei): Preprocess rustdoc links in mdbook or in the `operator_docgen` macro.
/// > Arguments: A Rust closure, the first argument is a received item and the
/// > second argument is a variadic [`var_args!` tuple list](https://hydro-project.github.io/hydroflow/doc/hydroflow/macro.var_args.html)
/// > where each item name is an output port.
///
/// Takes the input stream and allows the user to determine which items to
/// deliver to any number of output streams.
///
/// > Note: Downstream operators may need explicit type annotations.
///
/// > Note: The [`Pusherator`](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html)
/// > trait is automatically imported to enable the [`.give(...)` method](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html#tymethod.give).
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
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
pub const DEMUX_ENUM: OperatorConstraints = OperatorConstraints {
    name: "demux_enum",
    categories: &[OperatorCategory::MultiOut],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(2..),
    soft_range_out: &(2..),
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: &(0..1),
    is_external_input: false,
    ports_inn: None,
    ports_out: Some(|| PortListSpec::Variadic),
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    flow_prop_fn: None,
    write_fn: |&WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   outputs,
                   is_pull,
                   op_name,
                   op_inst:
                       OperatorInstance {
                           output_ports,
                           arguments,
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               diagnostics| {
        assert!(!is_pull);

        let enum_type = type_args
            .get(0)
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        // Port idents supplied via port connections in the surface syntax.
        let port_idents: Vec<_> = output_ports
            .iter()
            .filter_map(|output_port| {
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

                Some(port_ident)
            })
            .collect();

        let mut sort_permute: Vec<_> = (0..port_idents.len()).collect();
        sort_permute.sort_by_key(|&i| &port_idents[i]);

        let sorted_outputs = sort_permute.iter().map(|&i| &outputs[i]);

        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                #[allow(unused_imports)] use #root::pusherator::Pusherator;
                #root::pusherator::demux::Demux::new(
                    <#enum_type as #root::util::demux_enum::DemuxEnum::<_>>::demux_enum,
                    #root::var_expr!( #( #sorted_outputs ),* ),
                )
            };
        };

        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
