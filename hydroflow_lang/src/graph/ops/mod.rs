use std::fmt::Display;
use std::ops::{Bound, RangeBounds};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
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
    /// Generate code which runs once outside the subgraph to set up any
    /// external stuff like state API stuff or external chanels, etc.
    pub write_prologue_fn:
        &'static dyn Fn(&WriteContextArgs<'_>, &WriteIteratorArgs<'_>) -> TokenStream,
    /// Generate iterator or pusherator code inside the subgraphs.
    pub write_iterator_fn:
        &'static dyn Fn(&WriteContextArgs<'_>, &WriteIteratorArgs<'_>) -> TokenStream,
}

pub const RANGE_ANY: &'static dyn RangeTrait<usize> = &(0..);
pub const RANGE_0: &'static dyn RangeTrait<usize> = &(0..=0);
pub const RANGE_1: &'static dyn RangeTrait<usize> = &(1..=1);

const IDENTITY_WRITE_ITERATOR_FN: &'static dyn Fn(
    &WriteContextArgs<'_>,
    &WriteIteratorArgs<'_>,
) -> TokenStream = &(|&WriteContextArgs { ident, .. },
                      &WriteIteratorArgs {
                          inputs,
                          outputs,
                          is_pull,
                          ..
                      }| {
    if is_pull {
        let input = &inputs[0];
        quote! {
            let #ident = #input;
        }
    } else {
        let output = &outputs[0];
        quote! {
            let #ident = #output;
        }
    }
});

pub const OPERATORS: [OperatorConstraints; 20] = [
    OperatorConstraints {
        name: "null",
        hard_range_inn: RANGE_ANY,
        soft_range_inn: RANGE_ANY,
        hard_range_out: RANGE_ANY,
        soft_range_out: RANGE_ANY,
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  is_pull,
                                  ..
                              }| {
            if is_pull {
                quote! {
                    (#(#inputs.for_each(std::mem::drop)),*);
                    let #ident = std::iter::empty();
                }
            } else {
                quote! {
                    let _ = (#(#outputs),*);
                    let #ident = #root::pusherator::for_each::ForEach::new(std::mem::drop);
                }
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  is_pull,
                                  ..
                              }| {
            if is_pull {
                let mut inputs = inputs.iter();
                let first = inputs.next();
                let rest = inputs.map(|ident| quote! { .chain(#ident) });
                quote! {
                    let #ident = #first #( #rest )*;
                }
            } else {
                assert_eq!(1, outputs.len());
                let output = &outputs[0];
                quote! {
                    let #ident = #output;
                }
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
        write_prologue_fn: &(|wc, _| {
            // TODO(mingwei): use state api.
            let joindata_ident = wc.make_ident("joindata");
            quote! {
                let mut #joindata_ident = Default::default();
            }
        }),
        write_iterator_fn: &(|wc @ &WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs { inputs, .. }| {
            let joindata_ident = wc.make_ident("joindata");
            let lhs = &inputs[0];
            let rhs = &inputs[1];
            quote! {
                let #ident = #root::compiled::pull::SymmetricHashJoin::new(#lhs, #rhs, &mut #joindata_ident);
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  is_pull,
                                  ..
                              }| {
            if !is_pull {
                let tees = outputs
                    .iter()
                    .rev()
                    .map(|i| quote! { #i })
                    .reduce(|b, a| quote! { #root::pusherator::tee::Tee::new(#a, #b) })
                    .unwrap_or_else(|| quote! { std::iter::empty() });
                quote! {
                    let #ident = #tees;
                }
            } else {
                assert_eq!(1, inputs.len());
                let input = &inputs[0];
                quote! {
                    let #ident = #input;
                }
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: IDENTITY_WRITE_ITERATOR_FN,
    },
    OperatorConstraints {
        name: "map",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            if is_pull {
                let input = &inputs[0];
                quote! {
                    let #ident = #input.map(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote! {
                    let #ident = #root::pusherator::map::Map::new(#arguments, #output);
                }
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            if is_pull {
                let input = &inputs[0];
                quote! {
                    let #ident = #input.flat_map(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote! {
                    let #ident = #root::pusherator::map::Map::new(
                        #arguments,
                        #root::pusherator::flatten::Flatten::new(#output)
                    );
                }
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            if is_pull {
                let input = &inputs[0];
                quote! {
                    let #ident = #input.filter_map(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote! {
                    let #ident = #root::pusherator::filter_map::FilterMap::new(#arguments, #output);
                }
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  outputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            if is_pull {
                let input = &inputs[0];
                quote! {
                    let #ident = #input.filter(#arguments);
                }
            } else {
                let output = &outputs[0];
                quote! {
                    let #ident = #root::pusherator::filter::Filter::new(#arguments, #output);
                }
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            assert!(is_pull);
            let input = &inputs[0];
            // TODO(mingwei): Issues if initial value is not copy.
            // TODO(mingwei): Might introduce the initial value multiple times on scheduling.
            quote! {
                let #ident = std::iter::once(#input.fold(#arguments));
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            assert!(is_pull);
            let input = &inputs[0];
            quote! {
                let #ident = #input.reduce(#arguments).into_iter();
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs {
                                  inputs,
                                  arguments,
                                  is_pull,
                                  ..
                              }| {
            assert!(is_pull);
            let input = &inputs[0];
            quote! {
                // TODO(mingwei): unneccesary extra into_iter() then collect()
                let #ident = #input.collect::<std::collections::BinaryHeap<_>>(#arguments).into_sorted_vec().into_iter();
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
        write_prologue_fn: &(|wc, &WriteIteratorArgs { arguments, .. }| {
            let receiver = &arguments[0];
            let stream_ident = wc.make_ident("stream");
            quote! {
                let mut #stream_ident = Box::pin(#receiver);
            }
        }),
        write_iterator_fn: &(|wc @ &WriteContextArgs { root, ident, .. }, _| {
            let stream_ident = wc.make_ident("stream");
            quote! {
                let #ident = std::iter::from_fn(|| {
                    match #root::futures::stream::Stream::poll_next(#stream_ident.as_mut(), &mut std::task::Context::from_waker(&context.waker())) {
                        std::task::Poll::Ready(maybe) => maybe,
                        std::task::Poll::Pending => None,
                    }
                });
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
        write_prologue_fn: &(|wc, &WriteIteratorArgs { arguments, .. }| {
            let iter_ident = wc.make_ident("iter");
            quote! {
                let mut #iter_ident = std::iter::IntoIterator::into_iter(#arguments);
            }
        }),
        write_iterator_fn: &(|wc @ &WriteContextArgs { ident, .. }, _| {
            let iter_ident = wc.make_ident("iter");
            quote! {
                let #ident = #iter_ident.by_ref();
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs { arguments, .. }| {
            quote! {
                let #ident = std::iter::IntoIterator::into_iter(#arguments);
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
        write_prologue_fn: &(|wc @ &WriteContextArgs { root, .. }, _| {
            let handle_ident = wc.make_ident("diffdata_handle");
            quote! {
                let #handle_ident = df.add_state(std::cell::RefCell::new(
                    #root::lang::monotonic_map::MonotonicMap::<_, std::collections::HashSet<_>>::default(),
                ));
            }
        }),
        write_iterator_fn: &(|wc @ &WriteContextArgs { ident, .. },
                              &WriteIteratorArgs { inputs, .. }| {
            let handle_ident = wc.make_ident("diffdata_handle");

            let borrow_ident = wc.make_ident("borrow");
            let negset_ident = wc.make_ident("negset");

            let input_pos = &inputs[0];
            let input_neg = &inputs[1];
            quote! {
                let mut #borrow_ident = context.state_ref(#handle_ident).borrow_mut();
                let #negset_ident = #borrow_ident
                    .try_insert_with((context.current_epoch(), context.current_stratum()), || {
                        #input_neg.collect()
                    });
                let #ident = #input_pos.filter(move |x| !#negset_ident.contains(x));
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
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: IDENTITY_WRITE_ITERATOR_FN,
    },
    OperatorConstraints {
        name: "next_epoch",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| Some(DelayType::Epoch),
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: IDENTITY_WRITE_ITERATOR_FN,
    },
    OperatorConstraints {
        name: "for_each",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        num_args: 1,
        input_delaytype_fn: &|_| None,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs { arguments, .. }| {
            quote! {
                let #ident = #root::pusherator::for_each::ForEach::new(#arguments);
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
        write_prologue_fn: &(|wc @ &WriteContextArgs { root, .. },
                              &WriteIteratorArgs { arguments, .. }| {
            let async_write_arg = &arguments[0];

            let send_ident = wc.make_ident("item_send");
            let recv_ident = wc.make_ident("item_recv");

            quote! {
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
            }
        }),
        write_iterator_fn: &(|wc @ &WriteContextArgs { root, ident, .. }, _| {
            let send_ident = wc.make_ident("item_send");
            quote! {
                let #ident = #root::pusherator::for_each::ForEach::new(|item| {
                    #send_ident.send(item).expect("Failed to send async write item for processing.");
                });
            }
        }),
    },
];

pub struct WriteContextArgs<'a> {
    pub root: &'a TokenStream,
    pub subgraph_id: GraphSubgraphId,
    pub node_id: GraphNodeId,
    pub ident: &'a Ident,
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
    pub inputs: &'a [Ident],
    pub outputs: &'a [Ident],
    pub type_arguments: Option<&'a Punctuated<GenericArgument, Token![,]>>,
    pub arguments: &'a Punctuated<Expr, Token![,]>,
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
