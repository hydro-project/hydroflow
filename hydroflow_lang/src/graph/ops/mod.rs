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

pub const OPERATORS: [OperatorConstraints; 18] = [
    OperatorConstraints {
        name: "merge",
        hard_range_inn: &(0..),
        soft_range_inn: &(2..),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs { inputs, .. }| {
            let mut inputs = inputs.iter();
            let first = inputs.next();
            let rest = inputs.map(|ident| quote! { .chain(#ident) });
            quote! {
                let #ident = #first #( #rest )*;
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
        write_prologue_fn: &(|&WriteContextArgs {
                                  subgraph_id,
                                  node_id,
                                  ..
                              },
                              _| {
            // TODO(mingwei): use state api.
            let joindata_ident = Ident::new(
                &*format!(
                    "sg_{:?}_node_{:?}_joindata",
                    subgraph_id.data(),
                    node_id.data()
                ),
                Span::call_site(),
            );
            quote! {
                let mut #joindata_ident = Default::default();
            }
        }),
        write_iterator_fn: &(|&WriteContextArgs {
                                  root,
                                  subgraph_id,
                                  node_id,
                                  ident,
                                  ..
                              },
                              &WriteIteratorArgs { inputs, .. }| {
            let joindata_ident = Ident::new(
                &*format!(
                    "sg_{:?}_node_{:?}_joindata",
                    subgraph_id.data(),
                    node_id.data()
                ),
                Span::call_site(),
            );
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
        hard_range_out: &(0..),
        soft_range_out: &(2..),
        num_args: 0,
        input_delaytype_fn: &|_| None,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, ident, .. },
                              &WriteIteratorArgs { outputs, .. }| {
            let tees = outputs
                .iter()
                .rev()
                .map(|i| quote! { #i })
                .reduce(|b, a| quote! { #root::compiled::tee::Tee::new(#a, #b) })
                .unwrap_or_else(|| quote! { () }); // TODO(mingwei): this seems like it would cause an error?
            quote! {
                let #ident = #tees;
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
                    let #ident = #root::compiled::map::Map::new(#arguments, #output);
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
                    let #ident = #root::compiled::map::Map::new(
                        #arguments,
                        #root::compiled::flatten::Flatten::new(#output)
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
                    let #ident = #root::compiled::filter_map::FilterMap::new(#arguments, #output);
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
                    let #ident = #root::compiled::filter::Filter::new(#arguments, #output);
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
        write_prologue_fn: &(|_, &WriteIteratorArgs { arguments, .. }| {
            let receiver = &arguments[0];
            quote! {
                let mut #receiver = #receiver;
            }
        }),
        write_iterator_fn: &(|&WriteContextArgs { ident, .. },
                              &WriteIteratorArgs { arguments, .. }| {
            let receiver = &arguments[0];
            quote! {
                let #ident = std::iter::from_fn(|| {
                    match #receiver.poll_recv(&mut std::task::Context::from_waker(&mut context.waker())) {
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
        write_prologue_fn: &(|&WriteContextArgs {
                                  subgraph_id,
                                  node_id,
                                  ..
                              },
                              &WriteIteratorArgs { arguments, .. }| {
            let iter_ident = Ident::new(
                &*format!("sg_{:?}_node_{:?}_iter", subgraph_id.data(), node_id.data()),
                Span::call_site(),
            );
            quote! {
            let mut #iter_ident = std::iter::IntoIterator::into_iter(#arguments);
            }
        }),
        write_iterator_fn: &(|&WriteContextArgs {
                                  ident,
                                  subgraph_id,
                                  node_id,
                                  ..
                              },
                              _| {
            let iter_ident = Ident::new(
                &*format!("sg_{:?}_node_{:?}_iter", subgraph_id.data(), node_id.data()),
                Span::call_site(),
            );
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
                let #ident = #root::compiled::for_each::ForEach::new(#arguments);
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
        write_prologue_fn: &(|&WriteContextArgs {
                                  root,
                                  subgraph_id,
                                  node_id,
                                  ..
                              },
                              _| {
            let handle_ident = Ident::new(
                &*format!(
                    "sg_{:?}_node_{:?}_diffdata_handle",
                    subgraph_id.data(),
                    node_id.data()
                ),
                Span::call_site(),
            );
            quote! {
                let #handle_ident = df.add_state(std::cell::RefCell::new(
                    #root::lang::monotonic_map::MonotonicMap::<_, std::collections::HashSet<_>>::default(),
                ));
            }
        }),
        write_iterator_fn: &(|&WriteContextArgs {
                                  subgraph_id,
                                  node_id,
                                  ident,
                                  ..
                              },
                              &WriteIteratorArgs { inputs, .. }| {
            let handle_ident = Ident::new(
                &*format!(
                    "sg_{:?}_node_{:?}_diffdata_handle",
                    subgraph_id.data(),
                    node_id.data()
                ),
                Span::call_site(),
            );

            let borrow_ident = Ident::new(
                &*format!("node_{:?}_borrow", node_id.data()),
                Span::call_site(),
            );
            let negset_ident = Ident::new(
                &*format!("node_{:?}_negset", node_id.data()),
                Span::call_site(),
            );

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
];

pub struct WriteContextArgs<'a> {
    pub root: &'a TokenStream,
    pub subgraph_id: GraphSubgraphId,
    pub node_id: GraphNodeId,
    pub ident: &'a Ident,
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
