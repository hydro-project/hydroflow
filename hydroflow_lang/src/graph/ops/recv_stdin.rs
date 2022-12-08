use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 0 input streams, 1 output stream
///
/// > Arguments: port number
///
/// `recv_stdin` receives a Stream of lines from stdin
/// and emits each of the elements it receives downstream.
///
/// ```rustbook
/// let mut flow = hydroflow::hydroflow_syntax! {
///     recv_stdin() -> map(|x| x.unwrap().to_uppercase())
///         -> for_each(|x| println!("{}", x));
/// };
/// flow.run_async();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const RECV_STDIN: OperatorConstraints = OperatorConstraints {
    name: "recv_stdin",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs { ident, .. },
                 _| {
        let stream_ident = wc.make_ident("stream");
        let write_prologue = quote_spanned! {op_span=>
            let mut #stream_ident = {
                use tokio::io::AsyncBufReadExt;
                let reader = #root::tokio::io::BufReader::new(tokio::io::stdin());
                let stdin_lines = #root::tokio_stream::wrappers::LinesStream::new(reader.lines());
                Box::pin(stdin_lines)
            };
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
