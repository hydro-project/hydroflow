use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input streams, 1 output stream
///
/// > Arguments: The receive end of a tokio channel
///
/// Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
/// created in Rust code, `buffer`
/// is passed the receive endpoint of the channel and when receiving any element
/// will pass through all received inputs to the output unchanged.
///
/// ```rustbook
///     let timer = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
///         Duration::from_millis(1000),
///     ));
///
///     let mut df = hydroflow_syntax! {
///         repeat_iter(0..5) -> buffer(timer) -> for_each(|x| { println!("{x}"); });
///     };
///
///     df.run_async().await;
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const BUFFER: OperatorConstraints = OperatorConstraints {
    name: "buffer",
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     arguments,
                     inputs,
                     outputs,
                     is_pull,
                     ..
                 },
                 _| {
        let internal_buffer = wc.make_ident("internal_buffer");

        let receiver = &arguments[0];
        let stream_ident = wc.make_ident("stream");

        let write_prologue = quote_spanned! {op_span=>
            let mut #stream_ident = Box::pin(#receiver);
            let #internal_buffer = df.add_state(::std::cell::RefCell::new(::std::vec::Vec::new()));
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];

            quote_spanned! {op_span=>

                {
                    let mut vec = context.state_ref(#internal_buffer).borrow_mut();
                    vec.extend(#input);
                }

                let #ident = match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut std::task::Context::from_waker(&context.waker())) {
                    std::task::Poll::Ready(_) => {
                        let mut vec = context.state_ref(#internal_buffer).borrow_mut();
                        std::mem::replace(&mut *vec, Vec::new())
                    },
                    std::task::Poll::Pending => {
                        Vec::new()
                    },
                }.into_iter();
            }
        } else {
            let output = &outputs[0];

            quote_spanned! {op_span=>

                let #ident = hydroflow::pusherator::for_each::ForEach::new(|x| {
                    let mut vec = context.state_ref(#internal_buffer).borrow_mut();

                    vec.push(x);
                });

                {
                    let mut out = #output;
                    for x in match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut std::task::Context::from_waker(&context.waker())) {
                        std::task::Poll::Ready(_) => {
                            let mut vec = context.state_ref(#internal_buffer).borrow_mut();
                            std::mem::replace(&mut *vec, Vec::new())
                        },
                        std::task::Poll::Pending => {
                            Vec::new()
                        },
                    }.into_iter() {
                        use ::pusherator::Pusherator;
                        out.give(x);
                    }
                }
            }
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
