use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    OperatorCategory, OperatorConstraints, OperatorWriteOutput, PortListSpec, WriteContextArgs,
    RANGE_0, RANGE_1,
};
use crate::graph::{GraphEdgeType, OpInstGenerics, OperatorInstance, PortIndexValue};

// TODO(mingwei)
pub const STATE: OperatorConstraints = OperatorConstraints {
    name: "state",
    categories: &[OperatorCategory::Persistence],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(2..=2),
    soft_range_out: &(2..=2),
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: &(0..=1),
    is_external_input: false,
    ports_inn: None,
    ports_out: Some(|| PortListSpec::Fixed(parse_quote! { state, items })),
    input_delaytype_fn: |_| None,
    input_edgetype_fn: |_| Some(GraphEdgeType::Value),
    output_edgetype_fn: |idx| match idx {
        PortIndexValue::Path(path) if "state" == path.to_token_stream().to_string() => {
            GraphEdgeType::Reference
        }
        _else => GraphEdgeType::Value,
    },
    flow_prop_fn: None,
    write_fn: |&WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   op_inst:
                       OperatorInstance {
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               _diagnostics| {
        let generic_type = type_args
            .first()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(iter: Iter) -> impl ::std::iter::Iterator<Item = Item> { iter }
                    check_input::<_, #generic_type>(#input)
                };
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_output<Push: #root::pusherator::Pusherator<Item = Item>, Item>(push: Push) -> impl #root::pusherator::Pusherator<Item = Item> { push }
                    check_output::<_, #generic_type>(#output)
                };
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
