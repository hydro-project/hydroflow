use quote::ToTokens;

use super::{OperatorCategory, OperatorConstraints};
use crate::graph::{FlowProps, LatticeFlowType};

/// TODO(MINGWEI)
pub const CAST: OperatorConstraints = OperatorConstraints {
    name: "cast",
    categories: &[OperatorCategory::Map], // TODO(mingwei):?
    num_args: 1,
    flow_prop_fn: Some(|flow_props_in, op_inst, _star_ord| {
        assert_eq!(1, op_inst.input_ports.len());
        assert_eq!(1, op_inst.output_ports.len());
        let out_flow_type = match &*op_inst.arguments[0].to_token_stream().to_string() {
            "Some(LatticeFlowType::Delta)" => Some(LatticeFlowType::Delta),
            "Some(LatticeFlowType::Cumul)" => Some(LatticeFlowType::Cumul),
            "None" => None,
            _ => todo!("TODO(mingwei): diagnostics"),
        };
        // TODO(mingwei): diagnostics
        let in_props = flow_props_in[0].expect("TODO(mingwei): diagnostics");
        assert!(
            LatticeFlowType::can_downcast(in_props.lattice_flow_type, out_flow_type),
            "TODO(mingwei): diagnostics"
        );
        vec![Some(FlowProps {
            star_ord: in_props.star_ord,
            lattice_flow_type: out_flow_type,
        })]
    }),
    ..super::identity::IDENTITY
};
