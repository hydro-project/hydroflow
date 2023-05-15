use proc_macro2::Literal;
use quote::quote_spanned;

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};
use crate::graph::OperatorInstance;
use crate::pretty_span::PrettySpan;

/// > Arguments: An [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
///
/// Consumes items by sending them to an [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
/// A `Sink` is a thing into which values can be sent, asynchronously. For example, sending items
/// into a bounded channel.
///
/// Note this operator must be used within a Tokio runtime.
///
/// ```rustbook
/// # #[hydroflow::main]
/// # async fn main() {
/// // In this example we use a _bounded_ channel for our `Sink`. This is for demonstration only,
/// // instead you should use [`hydroflow::util::unbounded_channel`]. A bounded channel results in
/// // `Hydroflow` buffering items internally instead of within the channel. (We can't use
/// // unbounded here since unbounded channels are synchonous to write to and therefore not
/// // `Sink`s.)
/// let (send, recv) = tokio::sync::mpsc::channel::<usize>(5);
/// // `PollSender` adapts the send half of the bounded channel into a `Sink`.
/// let send = tokio_util::sync::PollSender::new(send);
///
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_iter(0..10) -> dest_sink(send);
/// };
/// // Call `run_async()` to allow async events to propegate, run for one second.
/// tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
///     .await
///     .expect_err("Expected time out");
///
/// let mut recv = tokio_stream::wrappers::ReceiverStream::new(recv);
/// // Only 5 elements received due to buffer size.
/// // (Note that if we were using a multi-threaded executor instead of `current_thread` it would
/// // be possible for more items to be added as they're removed, resulting in >5 collected.)
/// let out: Vec<_> = hydroflow::util::ready_iter(&mut recv).collect();
/// assert_eq!(&[0, 1, 2, 3, 4], &*out);
/// # }
/// ```
///
/// `Sink` is different from [`AsyncWrite`](https://docs.rs/futures/latest/futures/io/trait.AsyncWrite.html).
/// Instead of discrete values we send arbitrary streams of bytes into an `AsyncWrite` value. For
/// example, writings a stream of bytes to a file, a socket, or stdout.
///
/// To handle those situations we can use a codec from [`tokio_util::codec`](crate::tokio_util::codec).
/// These specify ways in which the byte stream is broken into individual items, such as with
/// newlines or with length delineation.
///
/// If we only want to write a stream of bytes without delineation we can use the [`BytesCodec`](crate::tokio_util::codec::BytesCodec).
///
/// In this example we use a [`duplex`](crate::tokio::io::duplex) as our `AsyncWrite` with a
/// `BytesCodec`.
///
/// ```rustbook
/// # #[hydroflow::main]
/// # async fn main() {
/// use bytes::Bytes;
/// use tokio::io::AsyncReadExt;
///
/// // Like a channel, but for a stream of bytes instead of discrete objects.
/// let (asyncwrite, mut asyncread) = tokio::io::duplex(256);
/// // Now instead handle discrete byte strings by length-encoding them.
/// let sink = tokio_util::codec::FramedWrite::new(asyncwrite, tokio_util::codec::BytesCodec::new());
///
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_iter([
///         Bytes::from_static(b"hello"),
///         Bytes::from_static(b"world"),
///     ]) -> dest_sink(sink);
/// };
/// tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
///     .await
///     .expect_err("Expected time out");
///
/// let mut buf = Vec::<u8>::new();
/// asyncread.read_buf(&mut buf).await.unwrap();
/// assert_eq!(b"helloworld", &*buf);
/// # }
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const DEST_SINK: OperatorConstraints = OperatorConstraints {
    name: "dest_sink",
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
                   is_pull,
                   op_name,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        const BUFFER_BACKPRESSURE_THRESHOLD: usize = 4096;

        assert!(!is_pull);

        let sink_arg = &arguments[0];

        let send_ident = wc.make_ident("item_send");
        let recv_ident = wc.make_ident("item_recv");
        let count_ident_send = wc.make_ident("count_send");
        let count_ident_recv = wc.make_ident("count_recv");
        let bp_ident_send = wc.make_ident("backpressure_valve_send");
        let bp_ident_recv = wc.make_ident("backpressure_valve_recv");
        let bp_threshold_lit = Literal::usize_suffixed(BUFFER_BACKPRESSURE_THRESHOLD);

        let sink_feed_msg = Literal::string(&*format!(
            "`{}()` ({}) encountered a error feeding async sink item.",
            op_name,
            PrettySpan(op_span)
        ));
        let sink_flush_msg = Literal::string(&*format!(
            "`{}()` ({}) encountered a error flushing async sink.",
            op_name,
            PrettySpan(op_span)
        ));
        let buffer_send_msg = Literal::string(&*format!(
            "`{}()` ({}) failed to send item into buffer channel (seeing this message constitutes a hydroflow bug): {{}}.",
            op_name,
            PrettySpan(op_span)
        ));

        let write_prologue = quote_spanned! {op_span=>
            let #count_ident_send = ::std::sync::Arc::new(::std::sync::atomic::AtomicUsize::new(0));
            let #count_ident_recv = ::std::sync::Arc::clone(&#count_ident_send);
            let (#send_ident, #recv_ident) = #root::tokio::sync::mpsc::unbounded_channel();
            let #bp_ident_send = #hydroflow.backpressure_valve();
            let #bp_ident_recv = ::std::clone::Clone::clone(&#bp_ident_send);
            {
                /// Function is needed so `Item` is so no ambiguity for what `Item` is used
                /// when calling `.flush()`.
                async fn sink_feed_flush<Sink, Item>(
                    mut recv: #root::tokio::sync::mpsc::UnboundedReceiver<Item>,
                    mut sink: Sink,
                    count: ::std::sync::Arc<::std::sync::atomic::AtomicUsize>,
                    backpressure_valve: #root::scheduled::context::BackpressureValve,
                ) where
                    Sink: ::std::marker::Unpin + #root::futures::Sink<Item>,
                    Sink::Error: ::std::fmt::Debug,
                {
                    while let Some(item) = recv.recv().await {
                        let mut recv_count = 1;
                        #root::futures::SinkExt::feed(&mut sink, item)
                            .await
                            .expect(#sink_feed_msg);
                        while let Ok(item) = recv.try_recv() {
                            recv_count += 1;
                            #root::futures::SinkExt::feed(&mut sink, item)
                                .await
                                .expect(#sink_feed_msg);
                        }
                        let old_count = count.fetch_sub(recv_count, ::std::sync::atomic::Ordering::Relaxed);
                        if #bp_threshold_lit <= old_count && old_count < #bp_threshold_lit + recv_count {
                            backpressure_valve.release(); // Release backpressure.
                        }

                        #root::futures::SinkExt::flush(&mut sink)
                            .await
                            .expect(#sink_flush_msg);
                    }
                }
                #hydroflow
                    .spawn_task(sink_feed_flush(#recv_ident, #sink_arg, #count_ident_recv, #bp_ident_recv));
            }
        };

        let write_iterator = quote_spanned! {op_span=>
            let #ident = #root::pusherator::for_each::ForEach::new(|item| {
                if let Err(err) = #send_ident.send(item) {
                    panic!(#buffer_send_msg, err);
                }
                let count = #count_ident_send.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
                if #bp_threshold_lit == 1 + count {
                    #bp_ident_send.trigger(); // Trigger backpressure.
                }
            });
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
