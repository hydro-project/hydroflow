use std::fmt::Display;
use std::ops::{Bound, RangeBounds};

// Don't use `quote!`, use `quote_spanned!`.
#[allow(unused_imports)]
use std::compile_error as quote;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use slotmap::Key;
use syn::punctuated::Punctuated;
use syn::{Expr, GenericArgument, Token};

use super::{GraphNodeId, GraphSubgraphId};

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum DelayType {
    Stratum,
    Epoch,
}

pub struct OperatorConstraints {
    /// Operator's name.
    pub name: &'static str,

    // TODO: generic argument ranges.
    /// Input argument range required to not show an error.
    pub hard_range_inn: &'static dyn RangeTrait<usize>,
    /// Input argument range required to not show a warning.
    pub soft_range_inn: &'static dyn RangeTrait<usize>,
    /// Output argument range required to not show an error.
    pub hard_range_out: &'static dyn RangeTrait<usize>,
    /// Output argument range required to not show an warning.
    pub soft_range_out: &'static dyn RangeTrait<usize>,
    /// Number of arguments i.e. `operator(a, b, c)` has `num_args = 3`.
    pub num_args: usize,

    /// Determines if this input must be preceeded by a stratum barrier.
    pub input_delaytype_fn: &'static dyn Fn(usize) -> Option<DelayType>,

    /// Emit code in multiple locations. See [`OperatorWriteOutput`].
    pub write_fn:
        &'static dyn Fn(&WriteContextArgs<'_>, &WriteIteratorArgs<'_>) -> OperatorWriteOutput,
}

#[derive(Default)]
#[non_exhaustive]
pub struct OperatorWriteOutput {
    /// Code which runs once outside the subgraph to set up any external stuff
    /// like state API stuff, external chanels, network connections, etc.
    pub write_prologue: TokenStream,
    /// Iterator (or pusherator) code inside the subgraphs. The code for each
    /// operator is emitted in order.
    ///
    /// Emitted code should assign to [`WriteContextArgs.ident`] and use
    /// [`WriteIteratorArgs.inputs`] (pull iterators) or
    /// [`WriteIteratorArgs.outputs`] (pusherators).
    pub write_iterator: TokenStream,
    /// Code which runs after iterators have been run. Mainly for flushing IO.
    pub write_iterator_after: TokenStream,
}

pub const RANGE_ANY: &'static dyn RangeTrait<usize> = &(0..);
pub const RANGE_0: &'static dyn RangeTrait<usize> = &(0..=0);
pub const RANGE_1: &'static dyn RangeTrait<usize> = &(1..=1);

pub fn identity_write_iterator_fn(
    write_context_args: &WriteContextArgs,
    write_iterator_args: &WriteIteratorArgs,
) -> TokenStream {
    let &WriteContextArgs { op_span, .. } = write_context_args;
    let &WriteIteratorArgs {
        ident,
        inputs,
        outputs,
        is_pull,
        ..
    } = write_iterator_args;
    if is_pull {
        let input = &inputs[0];
        quote_spanned! {op_span=>
            let #ident = #input;
        }
    } else {
        let output = &outputs[0];
        quote_spanned! {op_span=>
            let #ident = #output;
        }
    }
}

pub const IDENTITY_WRITE_FN: &'static dyn Fn(
    &WriteContextArgs<'_>,
    &WriteIteratorArgs<'_>,
) -> OperatorWriteOutput = &(|write_context_args, write_iterator_args| {
    let write_iterator = identity_write_iterator_fn(write_context_args, write_iterator_args);
    OperatorWriteOutput {
        write_iterator,
        ..Default::default()
    }
});

pub const OPERATORS: [OperatorConstraints; 21] = [
    OperatorConstraints {
        name: "null",
        hard_range_inn: RANGE_ANY,
        soft_range_inn: RANGE_ANY,
        hard_range_out: RANGE_ANY,
        soft_range_out: RANGE_ANY,
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if is_pull {
                quote_spanned! {op_span=>
                    (#(#inputs.for_each(std::mem::drop)),*);
                    let #ident = std::iter::empty();
                }
            } else {
                quote_spanned! {op_span=>
                    let _ = (#(#outputs),*);
                    let #ident = #root::pusherator::for_each::ForEach::new(std::mem::drop);
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "merge",
        hard_range_inn: RANGE_ANY,
        soft_range_inn: &(2..),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if is_pull {
                let chains = inputs
                    .iter()
                    .map(|i| i.to_token_stream())
                    .reduce(|a, b| quote_spanned! {op_span=> #a.chain(#b) })
                    .unwrap_or_else(|| quote_spanned! {op_span=> std::iter::empty() });
                quote_spanned! {op_span=>
                    let #ident = #chains;
                }
            } else {
                assert_eq!(1, outputs.len());
                let output = &outputs[0];
                quote_spanned! {op_span=>
                    let #ident = #output;
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "join",
        hard_range_inn: &(2..=2),
        soft_range_inn: &(2..=2),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs { ident, inputs, .. }| {
            let joindata_ident = wc.make_ident("joindata");
            let write_prologue = quote_spanned! {op_span=>
                let mut #joindata_ident = Default::default();
            };

            let lhs = &inputs[0];
            let rhs = &inputs[1];
            let write_iterator = quote_spanned! {op_span=>
                let #ident = #root::compiled::pull::SymmetricHashJoin::new(#lhs, #rhs, &mut #joindata_ident);
            };

            OperatorWriteOutput {
                write_prologue,
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "tee",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_ANY,
        soft_range_out: &(2..),
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if !is_pull {
                let tees = outputs
                    .iter()
                    .rev()
                    .map(|i| i.to_token_stream())
                    .reduce(|b, a| quote_spanned! {op_span=> #root::pusherator::tee::Tee::new(#a, #b) })
                    .unwrap_or_else(
                        || quote_spanned! {op_span=> #root::pusherator::for_each::ForEach::new(std::mem::drop) },
                    );
                quote_spanned! {op_span=>
                    let #ident = #tees;
                }
            } else {
                assert_eq!(1, inputs.len());
                let input = &inputs[0];
                quote_spanned! {op_span=>
                    let #ident = #input;
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "identity",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_fn: IDENTITY_WRITE_FN,
    },
    OperatorConstraints {
        name: "map",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if is_pull {
                let input = &inputs[0];
                quote_spanned! {op_span=>
                    let #ident = #input.map(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote_spanned! {op_span=>
                    let #ident = #root::pusherator::map::Map::new(#arguments, #output);
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "flat_map",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if is_pull {
                let input = &inputs[0];
                quote_spanned! {op_span=>
                    let #ident = #input.flat_map(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote_spanned! {op_span=>
                    let #ident = #root::pusherator::map::Map::new(
                        #arguments,
                        #root::pusherator::flatten::Flatten::new(#output)
                    );
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "filter_map",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if is_pull {
                let input = &inputs[0];
                quote_spanned! {op_span=>
                    let #ident = #input.filter_map(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote_spanned! {op_span=>
                    let #ident = #root::pusherator::filter_map::FilterMap::new(#arguments, #output);
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "filter",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         outputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            let write_iterator = if is_pull {
                let input = &inputs[0];
                quote_spanned! {op_span=>
                    let #ident = #input.filter(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote_spanned! {op_span=>
                    let #ident = #root::pusherator::filter::Filter::new(#arguments, #output);
                }
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "fold",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 2,
        input_delaytype_fn: &|_| Some(DelayType::Stratum),
        write_fn: &(|&WriteContextArgs { op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            assert!(is_pull);
            let input = &inputs[0];
            // TODO(mingwei): Issues if initial value is not copy.
            // TODO(mingwei): Might introduce the initial value multiple times on scheduling.
            let write_iterator = quote_spanned! {op_span=>
                let #ident = std::iter::once(#input.fold(#arguments));
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "reduce",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| Some(DelayType::Stratum),
        write_fn: &(|&WriteContextArgs { op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            assert!(is_pull);
            let input = &inputs[0];
            let write_iterator = quote_spanned! {op_span=>
                let #ident = #input.reduce(#arguments).into_iter();
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "sort",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| Some(DelayType::Stratum),
        write_fn: &(|&WriteContextArgs { op_span, .. },
                     &WriteIteratorArgs {
                         ident,
                         inputs,
                         arguments,
                         is_pull,
                         ..
                     }| {
            assert!(is_pull);
            let input = &inputs[0];
            let write_iterator = quote_spanned! {op_span=>
                // TODO(mingwei): unneccesary extra into_iter() then collect()
                let #ident = #input.collect::<std::collections::BinaryHeap<_>>(#arguments).into_sorted_vec().into_iter();
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "recv_stream",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident, arguments, ..
                     }| {
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
            OperatorWriteOutput {
                write_prologue,
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "recv_iter",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|wc @ &WriteContextArgs { op_span, .. },
                     &WriteIteratorArgs {
                         ident, arguments, ..
                     }| {
            let iter_ident = wc.make_ident("iter");
            let write_prologue = quote_spanned! {op_span=>
                let mut #iter_ident = std::iter::IntoIterator::into_iter(#arguments);
            };
            let write_iterator = quote_spanned! {op_span=>
                let #ident = #iter_ident.by_ref();
            };
            OperatorWriteOutput {
                write_prologue,
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "repeat_iter",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { op_span, .. },
                     &WriteIteratorArgs {
                         ident, arguments, ..
                     }| {
            let write_iterator = quote_spanned! {op_span=>
                let #ident = std::iter::IntoIterator::into_iter(#arguments);
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "difference",
        hard_range_inn: &(2..=2),
        soft_range_inn: &(2..=2),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|idx| (1 == idx).then_some(DelayType::Stratum),
        write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs { ident, inputs, .. }| {
            let handle_ident = wc.make_ident("diffdata_handle");
            let write_prologue = quote_spanned! {op_span=>
                let #handle_ident = df.add_state(std::cell::RefCell::new(
                    #root::lang::monotonic_map::MonotonicMap::<_, std::collections::HashSet<_>>::default(),
                ));
            };
            let write_iterator = {
                let borrow_ident = wc.make_ident("borrow");
                let negset_ident = wc.make_ident("negset");

                let input_pos = &inputs[0];
                let input_neg = &inputs[1];
                quote_spanned! {op_span=>
                    let mut #borrow_ident = context.state_ref(#handle_ident).borrow_mut();
                    let #negset_ident = #borrow_ident
                        .try_insert_with((context.current_epoch(), context.current_stratum()), || {
                            #input_neg.collect()
                        });
                    let #ident = #input_pos.filter(move |x| !#negset_ident.contains(x));
                }
            };
            OperatorWriteOutput {
                write_prologue,
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "next_stratum",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| Some(DelayType::Stratum),
        write_fn: IDENTITY_WRITE_FN,
    },
    OperatorConstraints {
        name: "next_epoch",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| Some(DelayType::Epoch),
        write_fn: IDENTITY_WRITE_FN,
    },
    OperatorConstraints {
        name: "for_each",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|&WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident, arguments, ..
                     }| {
            let write_iterator = quote_spanned! {op_span=>
                let #ident = #root::pusherator::for_each::ForEach::new(#arguments);
            };
            OperatorWriteOutput {
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "write_async",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident, arguments, ..
                     }| {
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
                    .expect("write_async() must be used within a tokio runtime");
            };

            let write_iterator = quote_spanned! {op_span=>
                let #ident = #root::pusherator::for_each::ForEach::new(|item| {
                    #send_ident.send(item).expect("Failed to send async write item for processing.");
                });
            };

            OperatorWriteOutput {
                write_prologue,
                write_iterator,
                ..Default::default()
            }
        }),
    },
    OperatorConstraints {
        name: "sink_async",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                     &WriteIteratorArgs {
                         ident, arguments, ..
                     }| {
            let sink_arg = &arguments[0];

            let send_ident = wc.make_ident("item_send");
            let recv_ident = wc.make_ident("item_recv");

            let write_prologue = quote_spanned! {op_span=>
                let (#send_ident, #recv_ident) = #root::tokio::sync::mpsc::unbounded_channel();
                df
                    .spawn_task(async move {
                        use #root::futures::sink::SinkExt;

                        let mut recv = #recv_ident;
                        let mut sink = #sink_arg;
                        while let Some(item) = recv.recv().await {
                            sink.feed(item).await.expect("Error processing async sink item.");
                            // Receive as many items synchronously as possible before flushing.
                            while let Ok(item) = recv.try_recv() {
                                sink.feed(item).await.expect("Error processing async sink item.");
                            }
                            sink.flush().await.expect("Failed to flush async sink.");
                        }
                    })
                    .expect("sink_async() must be used within a tokio runtime");
            };

            let write_iterator = quote_spanned! {op_span=>
                let #ident = #root::pusherator::for_each::ForEach::new(|item| {
                    #send_ident.send(item).expect("Failed to send async write item for processing.");
                });
            };

            OperatorWriteOutput {
                write_prologue,
                write_iterator,
                ..Default::default()
            }
        }),
    },
];

pub struct WriteContextArgs<'a> {
    pub root: &'a TokenStream,
    pub subgraph_id: GraphSubgraphId,
    pub node_id: GraphNodeId,
    pub op_span: Span,
}
impl WriteContextArgs<'_> {
    pub fn make_ident(&self, suffix: &'static str) -> Ident {
        Ident::new(
            &*format!(
                "sg_{:?}_node_{:?}_{}",
                self.subgraph_id.data(),
                self.node_id.data(),
                suffix
            ),
            Span::call_site(),
        )
    }
}

pub struct WriteIteratorArgs<'a> {
    /// Ident the iterator or pullerator should be assigned to.
    pub ident: &'a Ident,
    /// Input operator idents (used for pull).
    pub inputs: &'a [Ident],
    /// Output operator idents (used for push).
    pub outputs: &'a [Ident],
    /// Unused: Operator type arguments.
    pub type_arguments: Option<&'a Punctuated<GenericArgument, Token![,]>>,
    /// Arguments provided by the user into the operator as arguments.
    /// I.e. the `a, b, c` in `-> my_op(a, b, c) -> `.
    pub arguments: &'a Punctuated<Expr, Token![,]>,
    /// If a pull iterator (true) or pusherator (false) should be used.
    pub is_pull: bool,
}

pub trait RangeTrait<T>
where
    T: ?Sized,
{
    fn start_bound(&self) -> Bound<&T>;
    fn end_bound(&self) -> Bound<&T>;
    fn contains(&self, item: &T) -> bool
    where
        T: PartialOrd<T>;

    fn human_string(&self) -> String
    where
        T: Display + PartialEq,
    {
        match (self.start_bound(), self.end_bound()) {
            (Bound::Unbounded, Bound::Unbounded) => "any number of".to_owned(),

            (Bound::Included(n), Bound::Included(x)) if n == x => {
                format!("exactly {}", n)
            }
            (Bound::Included(n), Bound::Included(x)) => {
                format!("at least {} and at most {}", n, x)
            }
            (Bound::Included(n), Bound::Excluded(x)) => {
                format!("at least {} and less than {}", n, x)
            }
            (Bound::Included(n), Bound::Unbounded) => format!("at least {}", n),
            (Bound::Excluded(n), Bound::Included(x)) => {
                format!("more than {} and at most {}", n, x)
            }
            (Bound::Excluded(n), Bound::Excluded(x)) => {
                format!("more than {} and less than {}", n, x)
            }
            (Bound::Excluded(n), Bound::Unbounded) => format!("more than {}", n),
            (Bound::Unbounded, Bound::Included(x)) => format!("at most {}", x),
            (Bound::Unbounded, Bound::Excluded(x)) => format!("less than {}", x),
        }
    }
}

impl<R, T> RangeTrait<T> for R
where
    R: RangeBounds<T>,
{
    fn start_bound(&self) -> Bound<&T> {
        self.start_bound()
    }

    fn end_bound(&self) -> Bound<&T> {
        self.end_bound()
    }

    fn contains(&self, item: &T) -> bool
    where
        T: PartialOrd<T>,
    {
        self.contains(item)
    }
}
