use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > Arguments: An [`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html).
///
/// Consumes a stream of bytes (specifically `AsRef[u8]` items) by writing them
/// to an [`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html)
/// output.
///
/// Note this operator must be used within a Tokio runtime.
#[hydroflow_internalmacro::operator_docgen]
pub const DEST_ASYNCWRITE: OperatorConstraints = OperatorConstraints {
    name: "dest_asyncwrite",
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
        let async_write_arg = &arguments[0];

        let send_ident = wc.make_ident("item_send");
        let recv_ident = wc.make_ident("item_recv");

        let write_prologue = quote_spanned! {op_span=>
            let (#send_ident, #recv_ident) = #root::tokio::sync::mpsc::unbounded_channel();
            df
                .spawn_task(async move {
                    use #root::tokio::io::AsyncWriteExt;

                    let mut recv = #recv_ident;
                    let mut write = #async_write_arg;
                    while let Some(item) = recv.recv().await {
                        let bytes = std::convert::AsRef::<[u8]>::as_ref(&item);
                        write.write_all(bytes).await.expect("Error processing async write item.");
                    }
                })
                .expect("dest_asyncwrite() must be used within a Tokio runtime");
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
