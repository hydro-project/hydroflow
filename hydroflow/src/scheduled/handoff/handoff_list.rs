use ref_cast::RefCast;
use sealed::sealed;

use crate::scheduled::graph::HandoffData;
use crate::scheduled::port::{BaseCtx, BasePort};
use crate::scheduled::type_list::TypeList;
use crate::scheduled::SubgraphId;

use super::Handoff;

#[sealed]
pub trait SendPortList: BasePortList<true> {}
#[sealed]
impl<T: BasePortList<true>> SendPortList for T {}

#[sealed]
pub trait RecvPortList: BasePortList<false> {}
#[sealed]
impl<T: BasePortList<false>> RecvPortList for T {}

#[sealed]
pub trait BasePortList<const S: bool>: TypeList {
    fn set_graph_meta<'a>(
        &self,
        handoffs: &'a mut [HandoffData],
        pred: Option<SubgraphId>,
        succ: Option<SubgraphId>,
    );

    type Ctx<'a>: TypeList;
    fn make_ctx<'a>(&self, handoffs: &'a [HandoffData]) -> Self::Ctx<'a>;
}
#[sealed]
impl<Rest, H, const S: bool> BasePortList<S> for (BasePort<H, S>, Rest)
where
    H: Handoff,
    Rest: BasePortList<S>,
{
    fn set_graph_meta<'a>(
        &self,
        handoffs: &'a mut [HandoffData],
        pred: Option<SubgraphId>,
        succ: Option<SubgraphId>,
    ) {
        let (this, rest) = self;
        let handoff = handoffs.get_mut(this.handoff_id).unwrap();
        if let Some(pred) = pred {
            handoff.preds.push(pred);
        }
        if let Some(succ) = succ {
            handoff.succs.push(succ);
        }
        rest.set_graph_meta(handoffs, pred, succ);
    }

    type Ctx<'a> = (&'a BaseCtx<H, S>, Rest::Ctx<'a>);
    fn make_ctx<'a>(&self, handoffs: &'a [HandoffData]) -> Self::Ctx<'a> {
        let (this, rest) = self;
        let handoff = handoffs
            .get(this.handoff_id)
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
impl<const S: bool> BasePortList<S> for () {
    fn set_graph_meta<'a>(
        &self,
        _handoffs: &'a mut [HandoffData],
        _pred: Option<SubgraphId>,
        _succ: Option<SubgraphId>,
    ) {
    }

    type Ctx<'a> = ();
    fn make_ctx<'a>(&self, _handoffs: &'a [HandoffData]) -> Self::Ctx<'a> {}
}

/// A variadic list of Handoff types, represented using a lisp-style tuple structure.
///
/// This trait is sealed and not meant to be implemented or used directly. Instead tuple lists (which already implement this trait) should be used, for example:
/// ```ignore
/// type MyHandoffList = (VecHandoff<usize>, (VecHandoff<String>, (TeeingHandoff<u32>, ())));
/// ```
/// The [`tl!`] (tuple list) macro simplifies usage of this kind:
/// ```ignore
/// type MyHandoffList = tl!(VecHandoff<usize>, VecHandoff<String>, TeeingHandoff<u32>);
/// ```
#[sealed]
pub trait HandoffList: TypeList {
}
#[sealed]
impl<H, L> HandoffList for (H, L)
where
    H: 'static + Handoff,
    L: HandoffList,
{
}
#[sealed]
impl HandoffList for () {
}

// pub trait HandoffListSplit<A>: HandoffList
// where
//     A: HandoffList,
// {
//     type Suffix: HandoffList;

//     fn split_input_port(
//         input_port: Self::InputPort,
//     ) -> (A::InputPort, <Self::Suffix as HandoffList>::InputPort);

//     #[allow(clippy::needless_lifetimes)] // clippy false positive
//     fn split_recv_ctx<'a>(
//         recv_ctx: Self::RecvCtx<'a>,
//     ) -> (A::RecvCtx<'a>, <Self::Suffix as HandoffList>::RecvCtx<'a>);

//     fn split_output_port(
//         output_port: Self::OutputPort,
//     ) -> (A::OutputPort, <Self::Suffix as HandoffList>::OutputPort);

//     #[allow(clippy::needless_lifetimes)] // clippy false positive
//     fn split_send_ctx<'a>(
//         recv_ctx: Self::SendCtx<'a>,
//     ) -> (A::SendCtx<'a>, <Self::Suffix as HandoffList>::SendCtx<'a>);
// }

// impl<X, T, U> HandoffListSplit<(X, U)> for (X, T)
// where
//     X: Handoff,
//     T: HandoffListSplit<U>,
//     U: HandoffList,
// {
//     type Suffix = T::Suffix;

//     fn split_input_port(
//         input_port: Self::InputPort,
//     ) -> (
//         <(X, U) as HandoffList>::InputPort,
//         <Self::Suffix as HandoffList>::InputPort,
//     ) {
//         let (x, t) = input_port;
//         let (u, v) = <T as HandoffListSplit<U>>::split_input_port(t);
//         ((x, u), v)
//     }

//     #[allow(clippy::needless_lifetimes)]
//     fn split_recv_ctx<'a>(
//         recv_ctx: Self::RecvCtx<'a>,
//     ) -> (
//         <(X, U) as HandoffList>::RecvCtx<'a>,
//         <Self::Suffix as HandoffList>::RecvCtx<'a>,
//     ) {
//         let (x, t) = recv_ctx;
//         let (u, v) = <T as HandoffListSplit<U>>::split_recv_ctx(t);
//         ((x, u), v)
//     }

//     fn split_output_port(
//         output_port: Self::OutputPort,
//     ) -> (
//         <(X, U) as HandoffList>::OutputPort,
//         <Self::Suffix as HandoffList>::OutputPort,
//     ) {
//         let (x, t) = output_port;
//         let (u, v) = <T as HandoffListSplit<U>>::split_output_port(t);
//         ((x, u), v)
//     }

//     #[allow(clippy::needless_lifetimes)]
//     fn split_send_ctx<'a>(
//         send_ctx: Self::SendCtx<'a>,
//     ) -> (
//         <(X, U) as HandoffList>::SendCtx<'a>,
//         <Self::Suffix as HandoffList>::SendCtx<'a>,
//     ) {
//         let (x, t) = send_ctx;
//         let (u, v) = <T as HandoffListSplit<U>>::split_send_ctx(t);
//         ((x, u), v)
//     }
// }
// impl<T> HandoffListSplit<()> for T
// where
//     T: HandoffList,
// {
//     type Suffix = T;

//     fn split_input_port(
//         input_port: Self::InputPort,
//     ) -> (<() as HandoffList>::InputPort, T::InputPort) {
//         ((), input_port)
//     }

//     #[allow(clippy::needless_lifetimes)]
//     fn split_recv_ctx<'a>(
//         recv_ctx: Self::RecvCtx<'a>,
//     ) -> (<() as HandoffList>::RecvCtx<'a>, T::RecvCtx<'a>) {
//         ((), recv_ctx)
//     }

//     fn split_output_port(
//         output_port: Self::OutputPort,
//     ) -> (<() as HandoffList>::OutputPort, T::OutputPort) {
//         ((), output_port)
//     }

//     #[allow(clippy::needless_lifetimes)]
//     fn split_send_ctx<'a>(
//         send_ctx: Self::SendCtx<'a>,
//     ) -> (<() as HandoffList>::SendCtx<'a>, T::SendCtx<'a>) {
//         ((), send_ctx)
//     }
// }
