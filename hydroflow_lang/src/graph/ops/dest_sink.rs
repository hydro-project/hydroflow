use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > Arguments: An [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
///
/// Consumes items by sending them to an [async `Sink`](https://docs.rs/futures/latest/futures/sink/trait.Sink.html).
///
/// This handles a stream of individual items, of an arbitrary type, whereas [`dest_asyncwrite`](#dest_asyncwrite)
/// handles streams of bytes.
///
/// Note this operator must be used within a Tokio runtime.
///
/// ```rustbook
/// #[tokio::test]
/// async fn test_dest_sink() {
///     use bytes::Bytes;
///     use tokio::io::AsyncReadExt;
///     use tokio_util::codec;
///
///     // Like a channel, but for a stream of bytes instead of discrete objects.
///     let (asyncwrite, mut asyncread) = tokio::io::duplex(256);
///     // Now instead handle discrete byte lists by length-encoding them.
///     let mut sink = codec::LengthDelimitedCodec::builder()
///         .length_field_length(1)
///         .new_write(asyncwrite);
///
///     let mut flow = hydroflow::hydroflow_syntax! {
///         source_iter([
///             Bytes::from_static(b"hello"),
///             Bytes::from_static(b"world"),
///         ]) -> dest_sink(&mut sink);
///     };
///     tokio::time::timeout(std::time::Duration::from_secs(1), flow.run_async())
///         .await
///         .expect_err("Expected time out");
///
///     let mut buf = Vec::<u8>::new();
///     asyncread.read_buf(&mut buf).await.unwrap();
///     // `\x05` is length prefix of "5".
///     assert_eq!(b"\x05hello\x05world", &*buf);
/// }
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const DEST_SINK: OperatorConstraints = OperatorConstraints {
    name: "dest_sink",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_0,
    soft_range_out: RANGE_0,
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident, arguments, ..
                 },
                 _| {
        let sink_arg = &arguments[0];

        let send_ident = wc.make_ident("item_send");
        let recv_ident = wc.make_ident("item_recv");

        let write_prologue = quote_spanned! {op_span=>
            let (#send_ident, #recv_ident) = #root::tokio::sync::mpsc::unbounded_channel();
            df
                .spawn_task(async move {
                    use #root::futures::sink::SinkExt;

                    #[allow(unused_mut)] let mut recv = #recv_ident;
                    #[allow(unused_mut)] let mut sink = #sink_arg;
                    while let Some(item) = recv.recv().await {
                        sink.feed(item).await.expect("Error processing async sink item.");
                        // Receive as many items synchronously as possible before flushing.
                        while let Ok(item) = recv.try_recv() {
                            sink.feed(item).await.expect("Error processing async sink item.");
                        }
                        sink.flush().await.expect("Failed to flush async sink.");
                    }
                })
                .expect("dest_sink() must be used within a tokio runtime");
        };

        let write_iterator = quote_spanned! {op_span=>
            let #ident = #root::pusherator::for_each::ForEach::new(|item| {
                #send_ident.send(item).expect("Failed to send async write item for processing.");
            });
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
