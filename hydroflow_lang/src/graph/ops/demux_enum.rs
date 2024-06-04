use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{PathArguments, Token, Type, TypePath};

use super::{
    FlowPropArgs, OpInstGenerics, OperatorCategory, OperatorConstraints, OperatorInstance,
    OperatorWriteOutput, PortIndexValue, PortListSpec, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

/// > Generic Argument: A enum type which has `#[derive(DemuxEnum)]`. Must match the items in the input stream.
///
/// Takes an input stream of enum instances and splits them into their variants.
///
/// ```rustdoc
/// #[derive(DemuxEnum)]
/// enum Shape {
///     Square(f64),
///     Rectangle(f64, f64),
///     Circle { r: f64 },
///     Triangle { w: f64, h: f64 }
/// }
///
/// let mut df = hydroflow_syntax! {
///     my_demux = source_iter([
///         Shape::Square(9.0),
///         Shape::Rectangle(10.0, 8.0),
///         Shape::Circle { r: 5.0 },
///         Shape::Triangle { w: 12.0, h: 13.0 },
///     ]) -> demux_enum::<Shape>();
///
///     my_demux[Square] -> map(|s| s * s) -> out;
///     my_demux[Circle] -> map(|(r,)| std::f64::consts::PI * r * r) -> out;
///     my_demux[Rectangle] -> map(|(w, h)| w * h) -> out;
///     my_demux[Circle] -> map(|(w, h)| 0.5 * w * h) -> out;
///
///     out = union() -> for_each(|area| println!("Area: {}", area));
/// };
/// df.run_available();
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
    type_args: RANGE_1,
    is_external_input: false,
    has_singleton_output: false,
    ports_inn: None,
    ports_out: Some(|| PortListSpec::Variadic),
    input_delaytype_fn: |_| None,
    flow_prop_fn: Some(|FlowPropArgs { flow_props_in, .. }, _diagnostics| {
        // Preserve input flow properties.
        Ok(vec![flow_props_in[0]])
    }),
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
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               diagnostics| {
        assert!(!is_pull);

        let enum_type = &type_args[0];

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

        let enum_type_turbofish = ensure_turbofish(enum_type);
        let port_variant_check_match_arms = port_idents.iter().map(|port_ident| {
            quote_spanned! {port_ident.span()=>
                #enum_type_turbofish::#port_ident { .. } => ()
            }
        });

        let mut sort_permute: Vec<_> = (0..port_idents.len()).collect();
        sort_permute.sort_by_key(|&i| &port_idents[i]);

        let sorted_outputs = sort_permute.iter().map(|&i| &outputs[i]);

        // The entire purpose of this closure and match statement is to generate readable error messages:
        // "missing match arm: `Variant(_)` not covered."
        // Or "no variant named `Variant` found for enum `Shape`"
        // Note this uses the `enum_type`'s span.
        let write_prologue = quote_spanned! {enum_type.span()=>
            let _ = |__val: #enum_type| {
                fn check_impl_demux_enum<T: ?Sized + #root::util::demux_enum::DemuxEnumBase>(_: &T) {}
                check_impl_demux_enum(&__val);
                match __val {
                    #(
                        #port_variant_check_match_arms,
                    )*
                };
            };
        };
        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                let mut __outputs = ( #( #sorted_outputs, )* );
                #root::pusherator::for_each::ForEach::new(move |__item: #enum_type| {
                    #root::util::demux_enum::DemuxEnum::demux_enum(
                        __item,
                        &mut __outputs,
                    );
                })
            };
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};

/// Ensure enum type has double colon for turbofish syntax.
fn ensure_turbofish(ty: &Type) -> Type {
    let mut ty = ty.clone();
    if let Type::Path(TypePath { qself: _, path }) = &mut ty {
        if let Some(last_seg) = path.segments.last_mut() {
            if let PathArguments::AngleBracketed(angle_bracketed) = &mut last_seg.arguments {
                angle_bracketed.colon2_token = Some(<Token![::]>::default());
            }
        }
    };
    ty
}
