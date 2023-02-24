use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Bound, RangeBounds};

use once_cell::sync::OnceCell;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use slotmap::Key;
use syn::punctuated::Punctuated;
use syn::{Expr, GenericArgument, Token, Type};

use crate::diagnostic::Diagnostic;
use crate::parse::{Operator, PortIndex};

use super::{GraphNodeId, GraphSubgraphId, Node, PortIndexValue};
use serde::{Deserialize, Serialize};

mod anti_join;
mod cross_join;
mod demux;
mod dest_sink;
mod dest_sink_serde;
mod difference;
mod filter;
mod filter_map;
mod flat_map;
mod flatten;
mod fold;
mod for_each;
mod group_by;
mod identity;
mod inspect;
mod join;
mod map;
mod merge;
mod next_stratum;
mod next_tick;
mod null;
mod reduce;
mod repeat_iter;
mod sort;
mod sort_by;
mod source_iter;
mod source_stdin;
mod source_stream;
mod source_stream_serde;
mod tee;
mod unique;
mod unzip;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum DelayType {
    Stratum,
    Tick,
}

pub enum PortListSpec {
    Variadic,
    Fixed(Punctuated<PortIndex, Token![,]>),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum FlowPropertyVal {
    Yes,
    No,
    Preserve,
    CodeBlock,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct FlowProperties {
    pub deterministic: FlowPropertyVal,
    pub monotonic: FlowPropertyVal,
    pub tainted: bool,
}
impl Default for FlowProperties {
    fn default() -> Self {
        Self {
            deterministic: FlowPropertyVal::No,
            monotonic: FlowPropertyVal::No,
            tainted: false,
        }
    }
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
    /// How many persistence lifetime arguments can be provided.
    pub persistence_args: &'static dyn RangeTrait<usize>,
    // /// How many (non-persistence) lifetime arguemtns can be provided.
    // pub lifetime_args: &'static dyn RangeTrait<usize>,
    /// How many generic type arguments can be provided.
    pub type_args: &'static dyn RangeTrait<usize>,
    /// If this operator receives external inputs and therefore must be in
    /// stratum 0.
    pub is_external_input: bool,

    /// What named or numbered input ports to expect?
    pub ports_inn: Option<fn() -> PortListSpec>,
    /// What named or numbered output ports to expect?
    pub ports_out: Option<fn() -> PortListSpec>,

    /// Determines if this input must be preceeded by a stratum barrier.
    pub input_delaytype_fn: fn(&PortIndexValue) -> Option<DelayType>,

    /// Emit code in multiple locations. See [`OperatorWriteOutput`].
    pub write_fn: WriteFn,

    pub properties: FlowProperties,
}

pub type WriteFn = fn(
    &WriteContextArgs<'_>,
    &WriteIteratorArgs<'_>,
    &mut Vec<Diagnostic>,
) -> Result<OperatorWriteOutput, ()>;

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

pub const IDENTITY_WRITE_FN: WriteFn = |write_context_args, write_iterator_args, _| {
    let write_iterator = identity_write_iterator_fn(write_context_args, write_iterator_args);
    Ok(OperatorWriteOutput {
        write_iterator,
        ..Default::default()
    })
};

pub const OPERATORS: &[OperatorConstraints] = &[
    null::NULL,
    merge::MERGE,
    join::JOIN,
    cross_join::CROSS_JOIN,
    tee::TEE,
    unzip::UNZIP,
    identity::IDENTITY,
    map::MAP,
    inspect::INSPECT,
    flat_map::FLAT_MAP,
    flatten::FLATTEN,
    filter_map::FILTER_MAP,
    filter::FILTER,
    fold::FOLD,
    reduce::REDUCE,
    group_by::GROUP_BY,
    unique::UNIQUE,
    sort::SORT,
    sort_by::SORT_BY,
    source_iter::SOURCE_ITER,
    source_stdin::SOURCE_STDIN,
    source_stream::SOURCE_STREAM,
    source_stream_serde::SOURCE_STREAM_SERDE,
    repeat_iter::REPEAT_ITER,
    difference::DIFFERENCE,
    anti_join::ANTI_JOIN,
    next_stratum::NEXT_STRATUM,
    next_tick::NEXT_TICK,
    for_each::FOR_EACH,
    demux::DEMUX,
    dest_sink::DEST_SINK,
    dest_sink_serde::DEST_SINK_SERDE,
];
pub fn operator_lookup() -> &'static HashMap<&'static str, &'static OperatorConstraints> {
    pub static OPERATOR_LOOKUP: OnceCell<HashMap<&'static str, &'static OperatorConstraints>> =
        OnceCell::new();
    OPERATOR_LOOKUP.get_or_init(|| OPERATORS.iter().map(|op| (op.name, op)).collect())
}
pub fn find_node_op_constraints(node: &Node) -> Option<&'static OperatorConstraints> {
    if let Node::Operator(operator) = node {
        find_op_op_constraints(operator)
    } else {
        None
    }
}
pub fn find_op_op_constraints(operator: &Operator) -> Option<&'static OperatorConstraints> {
    let name = &*operator.name_string();
    operator_lookup().get(name).copied()
}

pub struct WriteContextArgs<'a> {
    /// `hydroflow` crate name for `use #root::something`.
    pub root: &'a TokenStream,
    /// `context` ident, the name of the provided
    /// [`hydroflow::scheduled::Context`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/context/struct.Context.html).
    pub context: &'a Ident,
    /// Subgraph ID in which this operator is contained.
    pub subgraph_id: GraphSubgraphId,
    /// Node ID identifying this operator in the flat or partitioned graph meta-datastructure.
    pub node_id: GraphNodeId,
    /// The source span of this operator.
    pub op_span: Span,
}
impl WriteContextArgs<'_> {
    pub fn make_ident(&self, suffix: impl AsRef<str>) -> Ident {
        Ident::new(
            &*format!(
                "sg_{:?}_node_{:?}_{}",
                self.subgraph_id.data(),
                self.node_id.data(),
                suffix.as_ref(),
            ),
            self.op_span,
        )
    }
}

pub struct WriteIteratorArgs<'a> {
    /// Ident the iterator or pullerator should be assigned to.
    pub ident: &'a Ident,
    /// If a pull iterator (true) or pusherator (false) should be used.
    pub is_pull: bool,
    /// Input operator idents (used for pull).
    pub inputs: &'a [Ident],
    /// Output operator idents (used for push).
    pub outputs: &'a [Ident],

    /// Port values used as this operator's input.
    pub input_ports: &'a [&'a PortIndexValue],
    /// Port values used as this operator's output.
    pub output_ports: &'a [&'a PortIndexValue],

    /// Operator generic (type or lifetime) arguments.
    pub generic_args: Option<&'a Punctuated<GenericArgument, Token![,]>>,
    /// Lifetime persistence arguments. Corresponds to a prefix of [`Self::generic_args`].
    pub persistence_args: &'a [Persistence],
    /// Type persistence arguments. Corersponds to a (suffix) of [`Self::generic_args`].
    pub type_args: &'a [&'a Type],
    /// Arguments provided by the user into the operator as arguments.
    /// I.e. the `a, b, c` in `-> my_op(a, b, c) -> `.
    pub arguments: &'a Punctuated<Expr, Token![,]>,
    /// Name of the operator (will match [`OperatorConstraints::name`]).
    pub op_name: &'static str,
}

pub trait RangeTrait<T>: Send + Sync
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
    R: RangeBounds<T> + Send + Sync,
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

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Persistence {
    Tick,
    Static,
}
