use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 0 input streams, 1 output stream
///
/// > Arguments: [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
///
/// Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
/// of (serialized payload, addr) pairs, deserializes the payload and emits each of the
/// elements it receives downstream.
///
/// ```rustbook
/// async fn serde_in() {
///     let (outbound, inbound) = hydroflow::util::bind_udp_socket("localhost:9000".into()).await;
///     let mut flow = hydroflow::hydroflow_syntax! {
///         recv_stream_serde(inbound) -> map(|(x, a): (String, std::net::SocketAddr)| x.to_uppercase())
///             -> for_each(|x| println!("{}", x));
///     };
///     flow.run_available();
/// }
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const RECV_STREAM_SERDE: OperatorConstraints = OperatorConstraints {
    name: "recv_stream_serde",
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
                    std::task::Poll::Ready(Some(std::result::Result::Ok((payload, addr)))) => Some((#root::util::deserialize_simple(payload), addr)),
                    std::task::Poll::Ready(Some(Err(_))) => None,
                    std::task::Poll::Ready(None) => None,
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
