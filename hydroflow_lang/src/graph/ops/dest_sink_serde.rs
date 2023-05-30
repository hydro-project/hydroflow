use super::{make_missing_runtime_msg, FlowProperties, FlowPropertyVal};

use super::{OperatorConstraints, OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1};

use quote::quote_spanned;

/// > Arguments: A [serializing async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
///
/// Consumes (payload, addr) pairs by serializing the payload and sending the resulting pair to an [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
///
/// Note this operator must be used within a Tokio runtime.
/// ```rustbook
/// async fn serde_out() {
///     let addr = hydroflow::util::ipv4_resolve("localhost:9000".into()).unwrap();
///     let (outbound, inbound, _) = hydroflow::util::bind_udp_bytes(addr).await;
///     let remote = hydroflow::util::ipv4_resolve("localhost:9001".into()).unwrap();
///     let mut flow = hydroflow::hydroflow_syntax! {
///         source_iter(vec![("hello".to_string(), 1), ("world".to_string(), 2)])
///             -> map (|m| (m, remote)) -> dest_sink_serde(outbound);
///     };
///     flow.run_available();
/// }
/// ```
pub const DEST_SINK_SERDE: OperatorConstraints = OperatorConstraints {
    name: "dest_sink_serde",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_0,
    soft_range_out: RANGE_0,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   op_name,
                   ..
               },
               diagnostics| {
        let missing_runtime_msg = make_missing_runtime_msg(op_name);

        let OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        } = (super::dest_sink::DEST_SINK.write_fn)(wc, diagnostics)?;

        let write_iterator = quote_spanned! {op_span=>
            ::std::debug_assert!(#root::tokio::runtime::Handle::try_current().is_ok(), #missing_runtime_msg);
            #write_iterator
            let #ident = #root::pusherator::map::Map::new(
                |(payload, addr)| (#root::util::serialize_to_bytes(payload), addr),
                #ident,
            );
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
