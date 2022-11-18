use std::fmt::Display;
use std::ops::{Bound, RangeBounds};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use slotmap::Key;
use syn::punctuated::Punctuated;
use syn::{Expr, GenericArgument, Token};

use crate::parse::PortIndex;

use super::{GraphNodeId, GraphSubgraphId, PortIndexValue};

mod cross_join;
mod difference;
mod filter;
mod filter_map;
mod flat_map;
mod flatten;
mod fold;
mod for_each;
mod groupby;
mod identity;
mod inspect;
mod join;
mod map;
mod merge;
mod next_epoch;
mod next_stratum;
mod null;
mod recv_iter;
mod recv_stream;
mod reduce;
mod repeat_iter;
mod sink_async;
mod sort;
mod tee;
mod unique;
mod write_async;

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

    /// What named or numbered input ports to expect?
    pub ports_inn: Option<&'static dyn Fn() -> Punctuated<PortIndex, Token![,]>>,
    /// What named or numbered output ports to expect?
    pub ports_out: Option<&'static dyn Fn() -> Punctuated<PortIndex, Token![,]>>,

    /// Determines if this input must be preceeded by a stratum barrier.
    pub input_delaytype_fn: &'static dyn Fn(&PortIndexValue) -> Option<DelayType>,

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

pub const OPERATORS: &[OperatorConstraints] = &[
    null::NULL,
    merge::MERGE,
    join::JOIN,
    cross_join::CROSS_JOIN,
    tee::TEE,
    identity::IDENTITY,
    map::MAP,
    inspect::INSPECT,
    flat_map::FLAT_MAP,
    flatten::FLATTEN,
    filter_map::FILTER_MAP,
    filter::FILTER,
    fold::FOLD,
    reduce::REDUCE,
    groupby::GROUPBY,
    unique::UNIQUE,
    sort::SORT,
    recv_stream::RECV_STREAM,
    recv_iter::RECV_ITER,
    repeat_iter::REPEAT_ITER,
    difference::DIFFERENCE,
    next_stratum::NEXT_STRATUM,
    next_epoch::NEXT_EPOCH,
    for_each::FOR_EACH,
    write_async::WRITE_ASYNC,
    sink_async::SINK_ASYNC,
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
