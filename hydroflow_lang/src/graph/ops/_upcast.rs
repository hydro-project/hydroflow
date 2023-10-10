use super::{FlowPropArgs, OperatorCategory, OperatorConstraints};
use crate::graph::FlowProps;

/// TODO(MINGWEI)
pub const _UPCAST: OperatorConstraints = OperatorConstraints {
    name: "_upcast",
    categories: &[OperatorCategory::Map], // TODO(mingwei):?
    num_args: 1,
    flow_prop_fn: Some(
        |fp @ FlowPropArgs {
             op_span, op_inst, ..
         },
         diagnostics| {
            assert_eq!(1, op_inst.input_ports.len());
            assert_eq!(1, op_inst.output_ports.len());

            let out_flow_type = super::cast::parse_flow_type(&op_inst.arguments[0], op_span)
                .map_err(|diagnostic| diagnostics.push(diagnostic))?;
            Ok(vec![Some(FlowProps {
                star_ord: fp.new_star_ord(),
                lattice_flow_type: out_flow_type,
            })])
        },
    ),
    ..super::identity::IDENTITY
};
