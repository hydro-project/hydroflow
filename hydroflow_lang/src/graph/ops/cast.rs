use proc_macro2::Span;
use quote::ToTokens;
use syn::Expr;

use super::{FlowPropArgs, FlowProps, LatticeFlowType, OperatorCategory, OperatorConstraints};
use crate::diagnostic::{Diagnostic, Level};

/// TODO(MINGWEI)
pub const CAST: OperatorConstraints = OperatorConstraints {
    name: "cast",
    categories: &[OperatorCategory::Map], // TODO(mingwei):?
    num_args: 1,
    flow_prop_fn: Some(
        |FlowPropArgs {
             op_span,
             op_name,
             op_inst,
             flow_props_in,
             ..
         },
         diagnostics| {
            assert_eq!(1, op_inst.input_ports.len());
            assert_eq!(1, op_inst.output_ports.len());

            let out_flow_type = parse_flow_type(&op_inst.arguments_pre[0], op_span)
                .map_err(|diagnostic| diagnostics.push(diagnostic))?;
            let Some(in_props) = flow_props_in[0] else {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    format!("Failed to determine flow properties for `{}` input, this may be a Hydroflow bug.", op_name),
                ));
                return Err(());
            };
            if !LatticeFlowType::can_downcast(in_props.lattice_flow_type, out_flow_type) {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    format!(
                        "Cannot ilegally up-cast from `{:?}` to `{:?}`.",
                        in_props.lattice_flow_type, out_flow_type
                    ),
                ));
                return Err(());
            }
            Ok(vec![Some(FlowProps {
                star_ord: in_props.star_ord,
                lattice_flow_type: out_flow_type,
            })])
        },
    ),
    ..super::identity::IDENTITY
};

/// Parse the flow type from an argument.
pub fn parse_flow_type(arg: &Expr, op_span: Span) -> Result<Option<LatticeFlowType>, Diagnostic> {
    let mut flow_type_str = arg.to_token_stream().to_string();
    flow_type_str.retain(|c| !c.is_whitespace());
    let out_flow_type = match &*flow_type_str {
        "Some(LatticeFlowType::Delta)" | "Some(Delta)" => Some(LatticeFlowType::Delta),
        "Some(LatticeFlowType::Cumul)" | "Some(Cumul)" => Some(LatticeFlowType::Cumul),
        "None" => None,
        unexpected => {
            return Err(Diagnostic::spanned(
                op_span,
                Level::Error,
                format!(
                    "Unknown value `{}`, expected one of: `{:?}`, `{:?}`, or `{:?}`.",
                    unexpected,
                    Some(LatticeFlowType::Delta),
                    Some(LatticeFlowType::Cumul),
                    <Option<LatticeFlowType>>::None,
                ),
            ));
        }
    };
    Ok(out_flow_type)
}
