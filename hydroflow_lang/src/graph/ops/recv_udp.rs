use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 0 input streams, 1 output stream
///
/// > Arguments: port number
///
/// `recv_net` binds to a local network port and receives a Stream of serialized data from a remote sender,
/// deserializes each element and emits each of the elements it receives downstream.
///
/// ```rustbook
/// let mut flow = hydroflow::hydroflow_syntax! {
///     recv_net(9000) -> map(|x| x.to_uppercase())
///         -> for_each(|x| println!("{}", x));
/// };
/// flow.run_async();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const RECV_UDP: OperatorConstraints = OperatorConstraints {
    name: "recv_udp",
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
        let port = &arguments[0];
        let stream_ident = wc.make_ident("stream");
        let write_prologue = quote_spanned! {op_span=>
            let mut #stream_ident = {
                use std::net::ToSocketAddrs;
                let mut addrs = format!("127.0.0.1:{}", #port)
                    .to_socket_addrs()
                    .unwrap();
                let addr = addrs.find(|addr| addr.is_ipv4()).expect("Unable to resolve connection address");
                let socket = UdpSocket::bind(addr).await.unwrap();
                let (_outbound, inbound) = hydroflow::util::udp_lines(socket);
                Box::pin(inbound)
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
