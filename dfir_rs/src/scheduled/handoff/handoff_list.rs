//! Module for variadic handoff port lists, [`PortList`].

use ref_cast::RefCast;
use sealed::sealed;
use variadics::{variadic_trait, Variadic};

use super::Handoff;
use crate::scheduled::graph::HandoffData;
use crate::scheduled::port::{Polarity, Port, PortCtx};
use crate::scheduled::{HandoffId, SubgraphId};

/// Sealed trait for variadic lists of ports.
///
/// See the [`variadics`] crate for the strategy we use to implement variadics in Rust.
#[sealed]
pub trait PortList<S>: Variadic
where
    S: Polarity,
{
    /// Iteratively/recursively set the graph metadata for each port in this list.
    ///
    /// Specifically sets:
    /// - `HandoffData::preds` and `HandoffData::succs` in the `handoffs` slice for the
    ///   handoffs in this [`PortList`] (using `pred` and/or `succ`).
    /// - `out_handoff_ids` will be extended with all the handoff IDs in this [`PortList`].
    ///
    /// `handoffs_are_preds`:
    /// - `true`: Handoffs are predecessors (inputs) to subgraph `sg_id`.
    /// - `false`: Handoffs are successors (outputs) from subgraph `sg_id`.
    fn set_graph_meta(
        &self,
        handoffs: &mut [HandoffData],
        out_handoff_ids: &mut Vec<HandoffId>,
        sg_id: SubgraphId,
        handoffs_are_preds: bool,
    );

    /// The [`Variadic`] return type of [`Self::make_ctx`].
    type Ctx<'a>: Variadic;
    /// Iteratively/recursively construct a `Ctx` variadic list.
    ///
    /// (Note that unlike [`Self::set_graph_meta`], this does not mess with pred/succ handoffs for
    /// teeing).
    fn make_ctx<'a>(&self, handoffs: &'a [HandoffData]) -> Self::Ctx<'a>;
}
#[sealed]
impl<S, Rest, H> PortList<S> for (Port<S, H>, Rest)
where
    S: Polarity,
    H: Handoff,
    Rest: PortList<S>,
{
    fn set_graph_meta(
        &self,
        handoffs: &mut [HandoffData],
        out_handoff_ids: &mut Vec<HandoffId>,
        sg_id: SubgraphId,
        handoffs_are_preds: bool,
    ) {
        let (this, rest) = self;
        let this_handoff = &mut handoffs[this.handoff_id.0];

        // Set subgraph's info (`out_handoff_ids`) about neighbor handoffs.
        // Use the "representative" handoff (pred or succ) for teeing handoffs, for the subgraph metadata.
        // For regular Vec handoffs, `pred_handoffs` and `succ_handoffs` will just be the handoff itself.
        out_handoff_ids.extend(if handoffs_are_preds {
            this_handoff.pred_handoffs.iter().copied()
        } else {
            this_handoff.succ_handoffs.iter().copied()
        });

        // Set handoff's info (`preds`/`succs`) about neighbor subgraph (`sg_id`).
        if handoffs_are_preds {
            for succ_hoff in this_handoff.succ_handoffs.clone() {
                handoffs[succ_hoff.0].succs.push(sg_id);
            }
        } else {
            for pred_hoff in this_handoff.pred_handoffs.clone() {
                handoffs[pred_hoff.0].preds.push(sg_id);
            }
        }
        rest.set_graph_meta(handoffs, out_handoff_ids, sg_id, handoffs_are_preds);
    }

    type Ctx<'a> = (&'a PortCtx<S, H>, Rest::Ctx<'a>);
    fn make_ctx<'a>(&self, handoffs: &'a [HandoffData]) -> Self::Ctx<'a> {
        let (this, rest) = self;
        let handoff = handoffs
            .get(this.handoff_id.0)
            .unwrap()
            .handoff
            .any_ref()
            .downcast_ref()
            .expect("Attempted to cast handoff to wrong type.");

        let ctx = RefCast::ref_cast(handoff);
        let ctx_rest = rest.make_ctx(handoffs);
        (ctx, ctx_rest)
    }
}
#[sealed]
impl<S> PortList<S> for ()
where
    S: Polarity,
{
    fn set_graph_meta(
        &self,
        _handoffs: &mut [HandoffData],
        _out_handoff_ids: &mut Vec<HandoffId>,
        _sg_id: SubgraphId,
        _handoffs_are_preds: bool,
    ) {
    }

    type Ctx<'a> = ();
    fn make_ctx<'a>(&self, _handoffs: &'a [HandoffData]) -> Self::Ctx<'a> {}
}

/// Trait for splitting a list of ports into two.
#[sealed]
pub trait PortListSplit<S, A>: PortList<S>
where
    S: Polarity,
    A: PortList<S>,
{
    /// The suffix, second half of the split.
    type Suffix: PortList<S>;

    /// Split the port list, returning the prefix and [`Self::Suffix`] as the two halves.
    fn split_ctx(ctx: Self::Ctx<'_>) -> (A::Ctx<'_>, <Self::Suffix as PortList<S>>::Ctx<'_>);
}
#[sealed]
impl<S, H, T, U> PortListSplit<S, (Port<S, H>, U)> for (Port<S, H>, T)
where
    S: Polarity,
    H: Handoff,
    T: PortListSplit<S, U>,
    U: PortList<S>,
{
    type Suffix = T::Suffix;

    fn split_ctx(
        ctx: Self::Ctx<'_>,
    ) -> (
        <(Port<S, H>, U) as PortList<S>>::Ctx<'_>,
        <Self::Suffix as PortList<S>>::Ctx<'_>,
    ) {
        let (x, t) = ctx;
        let (u, v) = T::split_ctx(t);
        ((x, u), v)
    }
}
#[sealed]
impl<S, T> PortListSplit<S, ()> for T
where
    S: Polarity,
    T: PortList<S>,
{
    type Suffix = T;

    fn split_ctx(ctx: Self::Ctx<'_>) -> ((), T::Ctx<'_>) {
        ((), ctx)
    }
}

variadic_trait! {
    /// A variadic list of Handoff types, represented using a lisp-style tuple structure.
    ///
    /// This trait is sealed and not meant to be implemented or used directly. Instead tuple lists (which already implement this trait) should be used, for example:
    /// ```ignore
    /// type MyHandoffList = (VecHandoff<usize>, (VecHandoff<String>, (TeeingHandoff<u32>, ())));
    /// ```
    /// The [`var_expr!`](crate::var) macro simplifies usage of this kind:
    /// ```ignore
    /// type MyHandoffList = var_expr!(VecHandoff<usize>, VecHandoff<String>, TeeingHandoff<u32>);
    /// ```
    #[sealed]
    pub variadic<T> HandoffList where T: 'static + Handoff {}
}
