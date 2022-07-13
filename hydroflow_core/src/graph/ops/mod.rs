use std::fmt::Display;
use std::ops::{Bound, RangeBounds};

use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub const RANGE_0: &'static dyn RangeTrait<usize> = &(0..=0);
pub const RANGE_1: &'static dyn RangeTrait<usize> = &(1..=1);

pub const OPERATORS: [OperatorConstraints; 8] = [
    OperatorConstraints {
        name: "merge",
        hard_range_inn: &(0..),
        soft_range_inn: &(2..),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|_, inputs, _, _| {
            let mut inputs = inputs.iter();
            let first = inputs.next();
            let rest = inputs.map(|ident| quote! { .chain(#ident) });
            quote! { #first #( #rest )* }
        }),
    },
    OperatorConstraints {
        name: "join",
        hard_range_inn: &(2..=2),
        soft_range_inn: &(2..=2),
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|root, inputs, _, _| {
            let lhs = &inputs[0];
            let rhs = &inputs[1];
            quote! {
                {
                    let mut todo = Default::default();
                    #root::compiled::pull::SymmetricHashJoin::new(#lhs, #rhs, &mut todo)
                }
            }
        }),
    },
    OperatorConstraints {
        name: "tee",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: &(0..),
        soft_range_out: &(2..),
        write_fn: &(|root, _, outputs, _| {
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
        write_fn: &(|_, inputs, _, args| {
            let input = &inputs[0];
            quote! { #input.map(#args) }
        }),
    },
    OperatorConstraints {
        name: "dedup",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|_, inputs, outputs, args| {
            let ts = quote! { dedup #( #inputs ),* #( #outputs ),* #args };
            let lit = Literal::string(&*format!("{}", ts));
            quote! { #lit }
        }),
    },
    OperatorConstraints {
        name: "input",
        hard_range_inn: RANGE_0,
        soft_range_inn: RANGE_0,
        hard_range_out: RANGE_1,
        soft_range_out: RANGE_1,
        write_fn: &(|root, _, _, _| {
            quote! {
                {
                    let (send, recv) = #root::tokio::sync::mpsc::unbounded_channel();
                    std::iter::from_fn(move || {
                        match recv.poll_recv(&mut std::task::Context::from_waker(&mut context.waker())) {
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
        write_fn: &(|_, _, _, args| {
            quote! { std::iter::IntoIterator::into_iter(#args) }
        }),
    },
    OperatorConstraints {
        name: "for_each",
        hard_range_inn: RANGE_1,
        soft_range_inn: RANGE_1,
        hard_range_out: RANGE_0,
        soft_range_out: RANGE_0,
        write_fn: &(|root, _inputs, _, args| {
            quote! { #root::compiled::for_each::ForEach::new(#args) }
        }),
    },
];

pub struct OperatorConstraints {
    pub name: &'static str,
    pub hard_range_inn: &'static dyn RangeTrait<usize>,
    pub soft_range_inn: &'static dyn RangeTrait<usize>,
    pub hard_range_out: &'static dyn RangeTrait<usize>,
    pub soft_range_out: &'static dyn RangeTrait<usize>,
    /// # Args
    /// 1. Root (`crate` or `hydroflow`)
    /// 2. Input identifiers.
    /// 3. Output identifiers.
    /// 4. Arguments.
    pub write_fn: &'static dyn Fn(
        &TokenStream,
        &[Ident],
        &[Ident],
        &Punctuated<Expr, Token![,]>,
    ) -> TokenStream,
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
