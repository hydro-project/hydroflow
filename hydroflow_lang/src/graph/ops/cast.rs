use quote::ToTokens;

use super::{FlowPropArgs, OperatorCategory, OperatorConstraints};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{FlowProps, LatticeFlowType};

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

            let out_flow_type = match &*op_inst.arguments[0].to_token_stream().to_string() {
                "Some(LatticeFlowType::Delta)" | "Some(Delta)" => Some(LatticeFlowType::Delta),
                "Some(LatticeFlowType::Cumul)" | "Some(Cumul)" => Some(LatticeFlowType::Cumul),
                "None" => None,
                unexpected => {
                    diagnostics.push(Diagnostic::spanned(
                        op_span,
                        Level::Error,
                        format!(
                            "Unknown value `{}`, expected one of: `{:?}`, `{:?}`, or `None`.",
                            unexpected,
                            Some(LatticeFlowType::Delta),
                            Some(LatticeFlowType::Cumul),
                        ),
                    ));
                    return Err(());
                }
            };
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
