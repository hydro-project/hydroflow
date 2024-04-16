use syn::Ident;

use super::{
    poll_futures::poll_futures_writer, FlowPropArgs, OperatorCategory, OperatorConstraints, RANGE_0, RANGE_1
};

pub const POLL_FUTURES_ORDERED: OperatorConstraints = OperatorConstraints {
    name: "poll_futures_ordered",
    categories: &[OperatorCategory::Map],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
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
    flow_prop_fn: Some(|FlowPropArgs { flow_props_in, .. }, _diagnostics| {
        // Preserve input flow properties.
        Ok(vec![flow_props_in[0]])
    }),
    write_fn: move |wc, _| poll_futures_writer(Ident::new("FuturesOrdered", wc.op_span),
    Ident::new("push_back", wc.op_span), 
    wc)
};
