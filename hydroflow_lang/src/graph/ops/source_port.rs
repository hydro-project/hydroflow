use crate::graph::OperatorInstance;

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 0 input streams, 1 output stream of `bytes::Bytes`
///
/// > Arguments: The logical name of an source port
///
/// The logical port name can be externally accessed outside of the hydroflow program and connected to a data source.
///
/// ```rustbook
/// async fn example() {
///     let mut df = hydroflow::hydroflow_syntax! {
///         source_port("source") -> for_each(|x| println!("{:?}", x));
///     };
///     let input = df.take_port_senders().remove("source").unwrap();
///     input.send(bytes::Bytes::from_static(b"hello")).await.unwrap();
///     df.run_available_async();
/// }
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const SOURCE_PORT: OperatorConstraints = OperatorConstraints {
    name: "source_port",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: true,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::DependsOnArgs,
        monotonic: FlowPropertyVal::DependsOnArgs,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   op_span,
                   ident,
                   hydroflow,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let receiver = &arguments[0];

        let stream_ident = wc.make_ident("stream");
        let write_prologue = quote_spanned! {op_span=>
            let mut #stream_ident = {
                let (tx, rx) = #root::util::unsync::mpsc::channel::<#root::bytes::Bytes>(None);
                #hydroflow.__add_in_port_sender(#receiver, tx);
                ::std::boxed::Box::pin(rx)
            };
        };
        let write_iterator = quote_spanned! {op_span=>
            let #ident = std::iter::from_fn(|| {
                match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut std::task::Context::from_waker(&#context.waker())) {
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
    },
};
