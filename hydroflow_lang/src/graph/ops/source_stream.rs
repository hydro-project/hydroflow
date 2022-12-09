use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 0 input streams, 1 output stream
///
/// > Arguments: The receive end of a tokio channel
///
/// Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
/// created in Rust code, `source_stream`
/// is passed the receive endpoint of the channel and emits each of the
/// elements it receives downstream.
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<&str>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv) -> map(|x| x.to_uppercase())
///         -> for_each(|x| println!("{}", x));
/// };
/// input_send.send("Hello").unwrap();
/// input_send.send("World").unwrap();
/// flow.run_available();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const SOURCE_STREAM: OperatorConstraints = OperatorConstraints {
    name: "source_stream",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident, arguments, ..
                 },
                 _| {
        let receiver = &arguments[0];
        let stream_ident = wc.make_ident("stream");
        let write_prologue = quote_spanned! {op_span=>
            let mut #stream_ident = Box::pin(#receiver);
        };
        let write_iterator = quote_spanned! {op_span=>
            let #ident = std::iter::from_fn(|| {
                match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut std::task::Context::from_waker(&context.waker())) {
                    std::task::Poll::Ready(maybe) => maybe,
                    std::task::Poll::Pending => None,
                }
            });
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
