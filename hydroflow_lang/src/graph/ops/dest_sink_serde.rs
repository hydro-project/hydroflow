use super::{make_missing_runtime_msg, FlowProperties, FlowPropertyVal};

use super::{
    OperatorConstraints, OperatorInstance, OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1,
};

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
#[hydroflow_internalmacro::operator_docgen]
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
                   hydroflow,
                   op_span,
                   ident,
                   op_name,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let sink_arg = &arguments[0];

        let send_ident: proc_macro2::Ident = wc.make_ident("item_send");
        let recv_ident = wc.make_ident("item_recv");
        let count_ident_send = wc.make_ident("count_send");
        let count_ident_recv = wc.make_ident("count_recv");

        let missing_runtime_msg = make_missing_runtime_msg(op_name);

        let write_prologue = quote_spanned! {op_span=>
            let #count_ident_send = ::std::sync::Arc::new(::std::sync::atomic::AtomicUsize::new(0));
            let #count_ident_recv = ::std::sync::Arc::clone(&#count_ident_send);
            let (#send_ident, #recv_ident) = #root::tokio::sync::mpsc::unbounded_channel();
            #hydroflow
                .spawn_task(async move {
                    use #root::futures::sink::SinkExt;

                    let mut recv = #recv_ident;
                    let mut sink = #sink_arg;
                    while let Some((payload, addr)) = recv.recv().await {
                        #count_ident_recv.fetch_sub(1, ::std::sync::atomic::Ordering::Relaxed);
                        let item = (#root::util::serialize_to_bytes(payload), addr);
                        sink.feed(item).await.expect("Error processing async sink item.");
                        // Receive as many items synchronously as possible before flushing.
                        while let Ok((payload, addr)) = recv.try_recv() {
                            #count_ident_recv.fetch_sub(1, ::std::sync::atomic::Ordering::Relaxed);

                            let item = (#root::util::serialize_to_bytes(payload), addr);
                            sink.feed(item).await.expect("Error processing async sink item.");
                        }
                        sink.flush().await.expect("Failed to flush async sink.");

                        let count = #count_ident_recv.load(::std::sync::atomic::Ordering::Relaxed);
                        println!("COUNT: {}", count);
                    }
                })
                .expect(#missing_runtime_msg);
        };

        let write_iterator = quote_spanned! {op_span=>
            let #ident = #root::pusherator::for_each::ForEach::new(|item| {
                if let Err(err) = #send_ident.send(item) {
                    panic!("Failed to send async write item for processing.: {}", err);
                }
                #count_ident_send.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
            });
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
