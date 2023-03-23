use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

use crate::graph::OperatorInstance;
use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// > Arguments: The receive end of a tokio channel that signals when to release the batch downstream.
///
/// Given a [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
/// created in Rust code, `batch`
/// is passed the receive end of the channel and when receiving any element
/// will pass through all received inputs to the output unchanged.
///
/// ```rustbook
///     let (tx, rx) = hydroflow::util::unbounded_channel::<()>();
///
///     // Will print 0, 1, 2, 3, 4 each on a new line just once.
///     let mut df = hydroflow::hydroflow_syntax! {
///         repeat_iter(0..5) -> batch(rx) -> for_each(|x| { println!("{x}"); });
///     };
///
///     tx.send(()).unwrap();
///
///     df.run_available();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const BATCH: OperatorConstraints = OperatorConstraints {
    name: "batch",
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
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::No,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   context,
                   hydroflow,
                   ident,
                   op_span,
                   root,
                   inputs,
                   outputs,
                   is_pull,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let internal_buffer = wc.make_ident("internal_buffer");

        let receiver = &arguments[0];
        let stream_ident = wc.make_ident("stream");

        let write_prologue = quote_spanned! {op_span=>
            let mut #stream_ident = ::std::boxed::Box::pin(#receiver);
            let #internal_buffer = #hydroflow.add_state(::std::cell::RefCell::new(::std::vec::Vec::new()));
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];

            quote_spanned! {op_span=>

                {
                    let mut vec = #context.state_ref(#internal_buffer).borrow_mut();
                    vec.extend(#input);
                }

                let #ident = match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut ::std::task::Context::from_waker(&context.waker())) {
                    ::std::task::Poll::Ready(_) => {
                        let mut vec = #context.state_ref(#internal_buffer).borrow_mut();
                        ::std::mem::take(&mut *vec)
                    },
                    ::std::task::Poll::Pending => {
                        ::std::vec::Vec::new()
                    },
                }.into_iter();
            }
        } else {
            let output = &outputs[0];

            quote_spanned! {op_span=>

                let #ident = #root::pusherator::for_each::ForEach::new(|x| {
                    let mut vec = #context.state_ref(#internal_buffer).borrow_mut();

                    vec.push(x);
                });

                {
                    let mut out = #output;
                    for x in match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut ::std::task::Context::from_waker(&context.waker())) {
                        ::std::task::Poll::Ready(_) => {
                            let mut vec = #context.state_ref(#internal_buffer).borrow_mut();
                            ::std::mem::take(&mut *vec)
                        },
                        ::std::task::Poll::Pending => {
                            ::std::vec::Vec::new()
                        },
                    }.into_iter() {
                        #root::pusherator::Pusherator::give(&mut out, x);
                    }
                }
            }
        };

        OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        }
    },
};
