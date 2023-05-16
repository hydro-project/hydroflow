use super::{FlowProperties, FlowPropertyVal};

use super::{OperatorConstraints, OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1};

use crate::graph::OperatorInstance;
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
///
/// > 1 input stream of `bytes::Bytes`, 0 output streams
///
/// > Arguments: The logical name of a destination port
///
/// The logical port name can be externally accessed outside of the hydroflow program and connected to a data sink.
///
/// ```rustbook
/// async fn example() {
///     let mut df = hydroflow::hydroflow_syntax! {
///         source_iter([bytes::Bytes::from_static(b"test")]) -> dest_port("dest");
///     };
///     let mut output = df.take_port_receivers().remove("dest").unwrap();
///     df.run_available_async();
///     assert_eq!(output.recv().await.unwrap(), bytes::Bytes::from_static(b"hello"));
/// }
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const DEST_PORT: OperatorConstraints = OperatorConstraints {
    name: "dest_port",
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
                   hydroflow,
                   op_inst: op_inst @ OperatorInstance { arguments, .. },
                   ..
               },
               diagnostics| {
        let port_name = &arguments[0];
        let stream_ident = wc.make_ident("stream");
        let prologue = quote_spanned!(op_span=>
            let #stream_ident = {
                let (tx, rx) = #root::tokio::sync::mpsc::channel(1000); // TODO: use unsync channel when unsync has an equivalent PollSender() Sink
                #hydroflow.__add_out_port_receiver(#port_name, rx);
                #root::tokio_util::sync::PollSender::new(tx)
            };
        );

        let mut punct = syn::punctuated::Punctuated::new();
        punct.push(syn::Expr::Verbatim(
            quote_spanned!(op_span=> #stream_ident ),
        ));

        let OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        } = (super::dest_sink::DEST_SINK.write_fn)(
            &WriteContextArgs {
                op_inst: &OperatorInstance {
                    arguments: punct,
                    ..op_inst.clone()
                },
                ..*wc
            },
            diagnostics,
        )?;

        let write_prologue = quote_spanned!(op_span=>
            #prologue
            #write_prologue
        );

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
