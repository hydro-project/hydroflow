use std::fmt::Display;
use std::ops::{Bound, RangeBounds};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use slotmap::Key;
use syn::punctuated::Punctuated;
use syn::{Expr, GenericArgument, Token};

use super::{NodeId, SubgraphId};

pub const RANGE_0: &'static dyn RangeTrait<usize> = &(0..=0);
pub const RANGE_1: &'static dyn RangeTrait<usize> = &(1..=1);

pub const OPERATORS: [OperatorConstraints; 7] = [
    OperatorConstraints {
        name: "merge",
        hard_range_inn: &(0..),
        soft_range_inn: &(2..),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|_, &WriteIteratorArgs { inputs, .. }| {
            let mut inputs = inputs.iter();
            let first = inputs.next();
            let rest = inputs.map(|ident| quote! { .chain(#ident) });
            quote! {
                #first #( #rest )*
            }
        }),
    },
    OperatorConstraints {
        name: "join",
        hard_range_inn: &(2..=2),
        soft_range_inn: &(2..=2),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
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
                #root::compiled::pull::SymmetricHashJoin::new(#lhs, #rhs, &mut #joindata_ident)
            }
        }),
    },
    OperatorConstraints {
        name: "tee",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: &(0..),
        soft_range_out: &(2..),
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, .. },
                              &WriteIteratorArgs { outputs, .. }| {
            outputs
                .iter()
                .rev()
                .map(|i| quote! { #i })
                .reduce(|b, a| quote! { #root::compiled::tee::Tee::new(#a, #b) })
                .unwrap_or_default()
        }),
    },
    OperatorConstraints {
        name: "map",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|_,
                              &WriteIteratorArgs {
                                  inputs, arguments, ..
                              }| {
            let input = &inputs[0];
            quote! { #input.map(#arguments) }
        }),
    },
    // OperatorConstraints {
    //     name: "dedup",
    //     hard_range_inn: RANGE_1,
    //     soft_range_inn: RANGE_1,
    //     hard_range_out: RANGE_1,
    //     soft_range_out: RANGE_1,
    //    write_prologue_fn: &(|_| quote! {}),
    //     write_fn: &(|_, inputs, outputs, args| {
    //         let ts = quote! { dedup #( #inputs ),* #( #outputs ),* #args };
    //         let lit = Literal::string(&*format!("{}", ts));
    //         quote! { #lit }
    //     }),
    // },
    OperatorConstraints {
        name: "input",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_prologue_fn:
            &(|&WriteContextArgs {
                   root,
                   subgraph_id,
                   node_id,
                   ..
               },
               &WriteIteratorArgs { type_arguments, .. }| {
                // TODO(mingwei): better span.
                let send_ident = Ident::new(
                    &*format!("sg_{:?}_node_{:?}_send", subgraph_id.data(), node_id.data()),
                    Span::call_site(),
                );
                let recv_ident = Ident::new(
                    &*format!("sg_{:?}_node_{:?}_recv", subgraph_id.data(), node_id.data()),
                    Span::call_site(),
                );
                quote! {
                    let (#send_ident, mut #recv_ident) = #root::tokio::sync::mpsc::unbounded_channel::<#type_arguments>();
                }
            }),
        write_iterator_fn: &(|&WriteContextArgs {
                                  subgraph_id,
                                  node_id,
                                  ..
                              },
                              _| {
            let recv_ident = Ident::new(
                &*format!("sg_{:?}_node_{:?}_recv", subgraph_id.data(), node_id.data()),
                Span::call_site(),
            );
            quote! {
                {
                    std::iter::from_fn(|| {
                        match #recv_ident.poll_recv(&mut std::task::Context::from_waker(&mut context.waker())) {
                            std::task::Poll::Ready(maybe) => maybe,
                            std::task::Poll::Pending => None,
                        }
                    })
                }
            }
        }),
    },
    OperatorConstraints {
        name: "seed",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|_, &WriteIteratorArgs { arguments, .. }| {
            quote! { std::iter::IntoIterator::into_iter(#arguments) }
        }),
    },
    OperatorConstraints {
        name: "for_each",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        write_prologue_fn: &(|_, _| quote! {}),
        write_iterator_fn: &(|&WriteContextArgs { root, .. },
                              &WriteIteratorArgs { arguments, .. }| {
            quote! { #root::compiled::for_each::ForEach::new(#arguments) }
        }),
    },
];

pub struct WriteContextArgs<'a> {
    pub root: &'a TokenStream,
    pub subgraph_id: SubgraphId,
    pub node_id: NodeId,
    pub ident: &'a Ident,
}

pub struct WriteIteratorArgs<'a> {
    pub inputs: &'a [Ident],
    pub outputs: &'a [Ident],
    pub type_arguments: Option<&'a Punctuated<GenericArgument, Token![,]>>,
    pub arguments: &'a Punctuated<Expr, Token![,]>,
}

pub struct OperatorConstraints {
    /// Operator's name.
    pub name: &'static str,
    /// Input argument range required to not show an error.
    pub hard_range_inn: &'static dyn RangeTrait<usize>,
    /// Input argument range required to not show a warning.
    pub soft_range_inn: &'static dyn RangeTrait<usize>,
    /// Output argument range required to not show an error.
    pub hard_range_out: &'static dyn RangeTrait<usize>,
    /// Output argument range required to not show an warning.
    pub soft_range_out: &'static dyn RangeTrait<usize>,
    // TODO: generic argument ranges.
    pub write_prologue_fn:
        &'static dyn Fn(&WriteContextArgs<'_>, &WriteIteratorArgs<'_>) -> TokenStream,
    pub write_iterator_fn:
        &'static dyn Fn(&WriteContextArgs<'_>, &WriteIteratorArgs<'_>) -> TokenStream,
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
