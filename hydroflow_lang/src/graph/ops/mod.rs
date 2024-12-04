//! Hydroflow's operators

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{Bound, RangeBounds};
use std::sync::OnceLock;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;
use serde::{Deserialize, Serialize};
use slotmap::Key;
use syn::punctuated::Punctuated;
use syn::{parse_quote_spanned, Expr, Token};

use super::{
    GraphNode, GraphNodeId, GraphSubgraphId, OpInstGenerics, OperatorInstance, PortIndexValue,
};
use crate::diagnostic::Diagnostic;
use crate::parse::{Operator, PortIndex};

/// The delay (soft barrier) type, for each input to an operator if needed.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum DelayType {
    /// Input must be collected over the preceeding stratum.
    Stratum,
    /// Monotone accumulation: can delay to reduce flow rate, but also correct to emit "early"
    MonotoneAccum,
    /// Input must be collected over the previous tick.
    Tick,
    /// Input must be collected over the previous tick but also not cause a new tick to occur.
    TickLazy,
}

/// Specification of the named (or unnamed) ports for an operator's inputs or outputs.
pub enum PortListSpec {
    /// Any number of unnamed (or optionally named) ports.
    Variadic,
    /// A specific number of named ports.
    Fixed(Punctuated<PortIndex, Token![,]>),
}

/// An instance of this struct represents a single hydroflow operator.
pub struct OperatorConstraints {
    /// Operator's name.
    pub name: &'static str,
    /// Operator categories, for docs.
    pub categories: &'static [OperatorCategory],

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
    // /// How many (non-persistence) lifetime arguments can be provided.
    // pub lifetime_args: &'static dyn RangeTrait<usize>,
    /// How many generic type arguments can be provided.
    pub type_args: &'static dyn RangeTrait<usize>,
    /// If this operator receives external inputs and therefore must be in
    /// stratum 0.
    pub is_external_input: bool,
    /// If this operator has a singleton reference output. For stateful operators.
    /// If true, [`WriteContextArgs::singleton_output_ident`] will be set to a meaningful value in
    /// the [`Self::write_fn`] invocation.
    pub has_singleton_output: bool,
    /// Flo semantics type.
    pub flo_type: Option<FloType>,

    /// What named or numbered input ports to expect?
    pub ports_inn: Option<fn() -> PortListSpec>,
    /// What named or numbered output ports to expect?
    pub ports_out: Option<fn() -> PortListSpec>,

    /// Determines if this input must be preceeded by a stratum barrier.
    pub input_delaytype_fn: fn(&PortIndexValue) -> Option<DelayType>,
    /// The operator's codegen. Returns code that is emited is several different locations. See [`OperatorWriteOutput`].
    pub write_fn: WriteFn,
}

/// Type alias for [`OperatorConstraints::write_fn`]'s type.
pub type WriteFn =
    fn(&WriteContextArgs<'_>, &mut Vec<Diagnostic>) -> Result<OperatorWriteOutput, ()>;

impl Debug for OperatorConstraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperatorConstraints")
            .field("name", &self.name)
            .field("hard_range_inn", &self.hard_range_inn)
            .field("soft_range_inn", &self.soft_range_inn)
            .field("hard_range_out", &self.hard_range_out)
            .field("soft_range_out", &self.soft_range_out)
            .field("num_args", &self.num_args)
            .field("persistence_args", &self.persistence_args)
            .field("type_args", &self.type_args)
            .field("is_external_input", &self.is_external_input)
            .field("ports_inn", &self.ports_inn)
            .field("ports_out", &self.ports_out)
            // .field("input_delaytype_fn", &self.input_delaytype_fn)
            // .field("flow_prop_fn", &self.flow_prop_fn)
            // .field("write_fn", &self.write_fn)
            .finish()
    }
}

/// The code generated and returned by a [`OperatorConstraints::write_fn`].
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

/// Convenience range: zero or more (any number).
pub const RANGE_ANY: &'static dyn RangeTrait<usize> = &(0..);
/// Convenience range: exactly zero.
pub const RANGE_0: &'static dyn RangeTrait<usize> = &(0..=0);
/// Convenience range: exactly one.
pub const RANGE_1: &'static dyn RangeTrait<usize> = &(1..=1);

/// Helper to write the `write_iterator` portion of [`OperatorConstraints::write_fn`] output for
/// unary identity operators.
pub fn identity_write_iterator_fn(
    &WriteContextArgs {
        root,
        op_span,
        ident,
        inputs,
        outputs,
        is_pull,
        op_inst:
            OperatorInstance {
                generics: OpInstGenerics { type_args, .. },
                ..
            },
        ..
    }: &WriteContextArgs,
) -> TokenStream {
    let generic_type = type_args
        .first()
        .map(quote::ToTokens::to_token_stream)
        .unwrap_or(quote_spanned!(op_span=> _));

    if is_pull {
        let input = &inputs[0];
        quote_spanned! {op_span=>
            let #ident = {
                fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(iter: Iter) -> impl ::std::iter::Iterator<Item = Item> { iter }
                check_input::<_, #generic_type>(#input)
            };
        }
    } else {
        let output = &outputs[0];
        quote_spanned! {op_span=>
            let #ident = {
                fn check_output<Push: #root::pusherator::Pusherator<Item = Item>, Item>(push: Push) -> impl #root::pusherator::Pusherator<Item = Item> { push }
                check_output::<_, #generic_type>(#output)
            };
        }
    }
}

/// [`OperatorConstraints::write_fn`] for unary identity operators.
pub const IDENTITY_WRITE_FN: WriteFn = |write_context_args, _| {
    let write_iterator = identity_write_iterator_fn(write_context_args);
    Ok(OperatorWriteOutput {
        write_iterator,
        ..Default::default()
    })
};

/// Helper to write the `write_iterator` portion of [`OperatorConstraints::write_fn`] output for
/// the null operator - an operator that ignores all inputs and produces no output.
pub fn null_write_iterator_fn(
    &WriteContextArgs {
        root,
        op_span,
        ident,
        inputs,
        outputs,
        is_pull,
        op_inst:
            OperatorInstance {
                generics: OpInstGenerics { type_args, .. },
                ..
            },
        ..
    }: &WriteContextArgs,
) -> TokenStream {
    let default_type = parse_quote_spanned! {op_span=> _};
    let iter_type = type_args.first().unwrap_or(&default_type);

    if is_pull {
        quote_spanned! {op_span=>
            #(
                #inputs.for_each(std::mem::drop);
            )*
            let #ident = std::iter::empty::<#iter_type>();
        }
    } else {
        quote_spanned! {op_span=>
            #[allow(clippy::let_unit_value)]
            let _ = (#(#outputs),*);
            let #ident = #root::pusherator::null::Null::<#iter_type>::new();
        }
    }
}

/// [`OperatorConstraints::write_fn`] for the null operator - an operator that ignores all inputs
/// and produces no output.
pub const NULL_WRITE_FN: WriteFn = |write_context_args, _| {
    let write_iterator = null_write_iterator_fn(write_context_args);
    Ok(OperatorWriteOutput {
        write_iterator,
        ..Default::default()
    })
};

macro_rules! declare_ops {
    ( $( $mod:ident :: $op:ident, )* ) => {
        $( pub(crate) mod $mod; )*
        /// All Hydroflow operators.
        pub const OPERATORS: &[OperatorConstraints] = &[
            $( $mod :: $op, )*
        ];
    };
}
declare_ops![
    all_once::ALL_ONCE,
    anti_join::ANTI_JOIN,
    anti_join_multiset::ANTI_JOIN_MULTISET,
    assert::ASSERT,
    assert_eq::ASSERT_EQ,
    batch::BATCH,
    chain::CHAIN,
    cross_join::CROSS_JOIN,
    cross_join_multiset::CROSS_JOIN_MULTISET,
    cross_singleton::CROSS_SINGLETON,
    demux::DEMUX,
    demux_enum::DEMUX_ENUM,
    dest_file::DEST_FILE,
    dest_sink::DEST_SINK,
    dest_sink_serde::DEST_SINK_SERDE,
    difference::DIFFERENCE,
    difference_multiset::DIFFERENCE_MULTISET,
    enumerate::ENUMERATE,
    filter::FILTER,
    filter_map::FILTER_MAP,
    flat_map::FLAT_MAP,
    flatten::FLATTEN,
    fold::FOLD,
    for_each::FOR_EACH,
    identity::IDENTITY,
    initialize::INITIALIZE,
    inspect::INSPECT,
    join::JOIN,
    join_fused::JOIN_FUSED,
    join_fused_lhs::JOIN_FUSED_LHS,
    join_fused_rhs::JOIN_FUSED_RHS,
    join_multiset::JOIN_MULTISET,
    fold_keyed::FOLD_KEYED,
    reduce_keyed::REDUCE_KEYED,
    lattice_bimorphism::LATTICE_BIMORPHISM,
    _lattice_fold_batch::_LATTICE_FOLD_BATCH,
    lattice_fold::LATTICE_FOLD,
    _lattice_join_fused_join::_LATTICE_JOIN_FUSED_JOIN,
    lattice_reduce::LATTICE_REDUCE,
    map::MAP,
    union::UNION,
    multiset_delta::MULTISET_DELTA,
    next_stratum::NEXT_STRATUM,
    defer_signal::DEFER_SIGNAL,
    defer_tick::DEFER_TICK,
    defer_tick_lazy::DEFER_TICK_LAZY,
    null::NULL,
    partition::PARTITION,
    persist::PERSIST,
    persist_mut::PERSIST_MUT,
    persist_mut_keyed::PERSIST_MUT_KEYED,
    py_udf::PY_UDF,
    reduce::REDUCE,
    spin::SPIN,
    sort::SORT,
    sort_by_key::SORT_BY_KEY,
    source_file::SOURCE_FILE,
    source_interval::SOURCE_INTERVAL,
    source_iter::SOURCE_ITER,
    source_json::SOURCE_JSON,
    source_stdin::SOURCE_STDIN,
    source_stream::SOURCE_STREAM,
    source_stream_serde::SOURCE_STREAM_SERDE,
    state::STATE,
    state_by::STATE_BY,
    tee::TEE,
    unique::UNIQUE,
    unzip::UNZIP,
    zip::ZIP,
    zip_longest::ZIP_LONGEST,
];

/// Get the operator lookup table, generating it if needed.
pub fn operator_lookup() -> &'static HashMap<&'static str, &'static OperatorConstraints> {
    pub static OPERATOR_LOOKUP: OnceLock<HashMap<&'static str, &'static OperatorConstraints>> =
        OnceLock::new();
    OPERATOR_LOOKUP.get_or_init(|| OPERATORS.iter().map(|op| (op.name, op)).collect())
}
/// Find an operator by [`GraphNode`].
pub fn find_node_op_constraints(node: &GraphNode) -> Option<&'static OperatorConstraints> {
    if let GraphNode::Operator(operator) = node {
        find_op_op_constraints(operator)
    } else {
        None
    }
}
/// Find an operator by an AST [`Operator`].
pub fn find_op_op_constraints(operator: &Operator) -> Option<&'static OperatorConstraints> {
    let name = &*operator.name_string();
    operator_lookup().get(name).copied()
}

/// Context arguments provided to [`OperatorConstraints::write_fn`].
#[derive(Clone)]
pub struct WriteContextArgs<'a> {
    /// `hydroflow` crate name for `use #root::something`.
    pub root: &'a TokenStream,
    /// `context` ident, the name of the provided
    /// [`hydroflow::scheduled::Context`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/context/struct.Context.html).
    pub context: &'a Ident,
    /// `df` ident, the name of the
    /// [`hydroflow::scheduled::graph::Hydroflow`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html)
    /// instance.
    pub hydroflow: &'a Ident,
    /// Subgraph ID in which this operator is contained.
    pub subgraph_id: GraphSubgraphId,
    /// Node ID identifying this operator in the flat or partitioned graph meta-datastructure.
    pub node_id: GraphNodeId,
    /// The source span of this operator.
    pub op_span: Span,

    /// Ident the iterator or pullerator should be assigned to.
    pub ident: &'a Ident,
    /// If a pull iterator (true) or pusherator (false) should be used.
    pub is_pull: bool,
    /// Input operator idents (or ref idents; used for pull).
    pub inputs: &'a [Ident],
    /// Output operator idents (or ref idents; used for push).
    pub outputs: &'a [Ident],
    /// Ident for the singleton output of this operator, if any.
    pub singleton_output_ident: &'a Ident,

    /// Operator name.
    pub op_name: &'static str,
    /// Operator instance arguments object.
    pub op_inst: &'a OperatorInstance,
    /// Arguments provided by the user into the operator as arguments.
    /// I.e. the `a, b, c` in `-> my_op(a, b, c) -> `.
    ///
    /// These arguments include singleton postprocessing codegen, with
    /// [`std::cell::RefCell::borrow_mut`] code pre-generated.
    pub arguments: &'a Punctuated<Expr, Token![,]>,
    /// Same as [`Self::arguments`] but with only `StateHandle`s, no borrowing code.
    pub arguments_handles: &'a Punctuated<Expr, Token![,]>,
}
impl WriteContextArgs<'_> {
    /// Generate a (almost certainly) unique identifier with the given suffix.
    ///
    /// Includes the subgraph and node IDs in the generated identifier.
    ///
    /// This will always return the same identifier for a given `suffix`.
    pub fn make_ident(&self, suffix: impl AsRef<str>) -> Ident {
        Ident::new(
            &format!(
                "sg_{:?}_node_{:?}_{}",
                self.subgraph_id.data(),
                self.node_id.data(),
                suffix.as_ref(),
            ),
            self.op_span,
        )
    }
}

/// An object-safe version of [`RangeBounds`].
pub trait RangeTrait<T>: Send + Sync + Debug
where
    T: ?Sized,
{
    /// Start (lower) bound.
    fn start_bound(&self) -> Bound<&T>;
    /// End (upper) bound.
    fn end_bound(&self) -> Bound<&T>;
    /// Returns if `item` is contained in this range.
    fn contains(&self, item: &T) -> bool
    where
        T: PartialOrd<T>;

    /// Turn this range into a human-readable string.
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
    R: RangeBounds<T> + Send + Sync + Debug,
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

/// Persistence lifetimes: `'tick`, `'static`, or `'mutable`.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Persistence {
    /// Persistence for one tick at-a-time only.
    Tick,
    /// Persistene across all ticks.
    Static,
    /// Mutability.
    Mutable,
}

/// Helper which creates a error message string literal for when the Tokio runtime is not found.
fn make_missing_runtime_msg(op_name: &str) -> Literal {
    Literal::string(&format!("`{}()` must be used within a Tokio runtime. For example, use `#[hydroflow::main]` on your main method.", op_name))
}

/// Operator categories, for docs.
///
/// See source of [`Self::description`] for description of variants.
#[allow(
    clippy::allow_attributes,
    missing_docs,
    reason = "see `Self::description`"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatorCategory {
    Map,
    Filter,
    Flatten,
    Fold,
    KeyedFold,
    LatticeFold,
    Persistence,
    MultiIn,
    MultiOut,
    Source,
    Sink,
    Control,
    CompilerFusionOperator,
    Windowing,
    Unwindowing,
}
impl OperatorCategory {
    /// Human-readible heading name, for docs.
    pub fn name(self) -> &'static str {
        match self {
            OperatorCategory::Map => "Maps",
            OperatorCategory::Filter => "Filters",
            OperatorCategory::Flatten => "Flattens",
            OperatorCategory::Fold => "Folds",
            OperatorCategory::KeyedFold => "Keyed Folds",
            OperatorCategory::LatticeFold => "Lattice Folds",
            OperatorCategory::Persistence => "Persistent Operators",
            OperatorCategory::MultiIn => "Multi-Input Operators",
            OperatorCategory::MultiOut => "Multi-Output Operators",
            OperatorCategory::Source => "Sources",
            OperatorCategory::Sink => "Sinks",
            OperatorCategory::Control => "Control Flow Operators",
            OperatorCategory::CompilerFusionOperator => "Compiler Fusion Operators",
            OperatorCategory::Windowing => "Windowing Operator",
            OperatorCategory::Unwindowing => "Un-Windowing Operator",
        }
    }
    /// Human description, for docs.
    pub fn description(self) -> &'static str {
        match self {
            OperatorCategory::Map => "Simple one-in-one-out operators.",
            OperatorCategory::Filter => "One-in zero-or-one-out operators.",
            OperatorCategory::Flatten => "One-in multiple-out operators.",
            OperatorCategory::Fold => "Operators which accumulate elements together.",
            OperatorCategory::KeyedFold => "Keyed fold operators.",
            OperatorCategory::LatticeFold => "Folds based on lattice-merge.",
            OperatorCategory::Persistence => "Persistent (stateful) operators.",
            OperatorCategory::MultiIn => "Operators with multiple inputs.",
            OperatorCategory::MultiOut => "Operators with multiple outputs.",
            OperatorCategory::Source => {
                "Operators which produce output elements (and consume no inputs)."
            }
            OperatorCategory::Sink => {
                "Operators which consume input elements (and produce no outputs)."
            }
            OperatorCategory::Control => "Operators which affect control flow/scheduling.",
            OperatorCategory::CompilerFusionOperator => {
                "Operators which are necessary to implement certain optimizations and rewrite rules"
            }
            OperatorCategory::Windowing => "Operators for windowing `loop` inputs.",
            OperatorCategory::Unwindowing => "Operators for collecting `loop` outputs.",
        }
    }
}

/// Operator type for Flo semantics.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum FloType {
    /// A source operator, which must be at the top level.
    Source,
    /// A windowing operator, for moving data into a loop context.
    Windowing,
    /// An un-windowing operator, for moving data out of a loop context.
    Unwindowing,
}
