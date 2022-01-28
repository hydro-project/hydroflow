use ref_cast::RefCast;
use sealed::sealed;

use crate::scheduled::ctx::{InputPort, OutputPort, RecvCtx, SendCtx};
use crate::scheduled::graph::HandoffData;
use crate::scheduled::type_list::TypeList;
use crate::scheduled::SubgraphId;

use super::Handoff;

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
    type OutputPort: TypeList;
    fn set_succs<'a>(
        sg_id: SubgraphId,
        handoffs: &'a mut [HandoffData],
        recv_ports: &Self::OutputPort,
    );

    type RecvCtx<'a>: TypeList;
    fn make_recv<'a>(
        handoffs: &'a [HandoffData],
        recv_ports: &Self::OutputPort,
    ) -> Self::RecvCtx<'a>;

    type InputPort: TypeList;
    fn set_preds<'a>(
        sg_id: SubgraphId,
        handoffs: &'a mut [HandoffData],
        send_ports: &Self::InputPort,
    );

    type SendCtx<'a>: TypeList;
    fn make_send<'a>(
        handoffs: &'a [HandoffData],
        send_ports: &Self::InputPort,
    ) -> Self::SendCtx<'a>;
}
#[sealed]
impl<H, L> HandoffList for (H, L)
where
    H: 'static + Handoff,
    L: HandoffList,
{
    type OutputPort = (OutputPort<H>, L::OutputPort);
    fn set_succs<'a>(
        sg_id: SubgraphId,
        handoffs: &'a mut [HandoffData],
        recv_ports: &Self::OutputPort,
    ) {
        let (hid, hid_rest) = recv_ports;

        handoffs.get_mut(hid.handoff_id).unwrap().succs.push(sg_id);
        L::set_succs(sg_id, handoffs, hid_rest);
    }

    type RecvCtx<'a> = (&'a RecvCtx<H>, L::RecvCtx<'a>);
    fn make_recv<'a>(
        handoffs: &'a [HandoffData],
        recv_ports: &Self::OutputPort,
    ) -> Self::RecvCtx<'a> {
        let (hid, hid_rest) = recv_ports;
        let hid = hid.handoff_id;
        let handoff = handoffs
            .get(hid)
            .unwrap()
            .handoff
            .any_ref()
            .downcast_ref()
            .expect("Attempted to cast handoff to wrong type.");
        let ctx = RefCast::ref_cast(handoff);

        let ctx_rest = L::make_recv(handoffs, hid_rest);
        (ctx, ctx_rest)
    }

    type InputPort = (InputPort<H>, L::InputPort);
    fn set_preds<'a>(
        sg_id: SubgraphId,
        handoffs: &'a mut [HandoffData],
        send_ports: &Self::InputPort,
    ) {
        let (hid, hid_rest) = send_ports;

        handoffs.get_mut(hid.handoff_id).unwrap().preds.push(sg_id);
        L::set_preds(sg_id, handoffs, hid_rest);
    }

    type SendCtx<'a> = (&'a SendCtx<H>, L::SendCtx<'a>);
    fn make_send<'a>(
        handoffs: &'a [HandoffData],
        send_ports: &Self::InputPort,
    ) -> Self::SendCtx<'a> {
        let (hid, hid_rest) = send_ports;
        let hid = hid.handoff_id;
        let handoff = handoffs
            .get(hid)
            .unwrap()
            .handoff
            .any_ref()
            .downcast_ref()
            .expect("Attempted to cast handoff to wrong type.");
        let ctx = RefCast::ref_cast(handoff);

        let ctx_rest = L::make_send(handoffs, hid_rest);
        (ctx, ctx_rest)
    }
}
#[sealed]
impl HandoffList for () {
    type OutputPort = ();
    fn set_succs<'a>(
        _sg_id: SubgraphId,
        _handoffs: &'a mut [HandoffData],
        _recv_ports: &Self::OutputPort,
    ) {
    }

    type RecvCtx<'a> = ();
    fn make_recv<'a>(
        _handoffs: &'a [HandoffData],
        _recv_ports: &Self::InputPort,
    ) -> Self::RecvCtx<'a> {
    }

    type InputPort = ();
    fn set_preds<'a>(
        _sg_id: SubgraphId,
        _handoffs: &'a mut [HandoffData],
        _send_ports: &Self::InputPort,
    ) {
    }

    type SendCtx<'a> = ();
    fn make_send<'a>(
        _handoffs: &'a [HandoffData],
        _send_ports: &Self::OutputPort,
    ) -> Self::SendCtx<'a> {
    }
}

pub trait HandoffListSplit<A>: HandoffList
where
    A: HandoffList,
{
    type Suffix: HandoffList;

    fn split_input_port(
        input_port: Self::InputPort,
    ) -> (A::InputPort, <Self::Suffix as HandoffList>::InputPort);

    #[allow(clippy::needless_lifetimes)] // clippy false positive
    fn split_recv_ctx<'a>(
        recv_ctx: Self::RecvCtx<'a>,
    ) -> (A::RecvCtx<'a>, <Self::Suffix as HandoffList>::RecvCtx<'a>);

    fn split_output_port(
        output_port: Self::OutputPort,
    ) -> (A::OutputPort, <Self::Suffix as HandoffList>::OutputPort);

    #[allow(clippy::needless_lifetimes)] // clippy false positive
    fn split_send_ctx<'a>(
        recv_ctx: Self::SendCtx<'a>,
    ) -> (A::SendCtx<'a>, <Self::Suffix as HandoffList>::SendCtx<'a>);
}

impl<X, T, U> HandoffListSplit<(X, U)> for (X, T)
where
    X: Handoff,
    T: HandoffListSplit<U>,
    U: HandoffList,
{
    type Suffix = T::Suffix;

    fn split_input_port(
        input_port: Self::InputPort,
    ) -> (
        <(X, U) as HandoffList>::InputPort,
        <Self::Suffix as HandoffList>::InputPort,
    ) {
        let (x, t) = input_port;
        let (u, v) = <T as HandoffListSplit<U>>::split_input_port(t);
        ((x, u), v)
    }

    #[allow(clippy::needless_lifetimes)]
    fn split_recv_ctx<'a>(
        recv_ctx: Self::RecvCtx<'a>,
    ) -> (
        <(X, U) as HandoffList>::RecvCtx<'a>,
        <Self::Suffix as HandoffList>::RecvCtx<'a>,
    ) {
        let (x, t) = recv_ctx;
        let (u, v) = <T as HandoffListSplit<U>>::split_recv_ctx(t);
        ((x, u), v)
    }

    fn split_output_port(
        output_port: Self::OutputPort,
    ) -> (
        <(X, U) as HandoffList>::OutputPort,
        <Self::Suffix as HandoffList>::OutputPort,
    ) {
        let (x, t) = output_port;
        let (u, v) = <T as HandoffListSplit<U>>::split_output_port(t);
        ((x, u), v)
    }

    #[allow(clippy::needless_lifetimes)]
    fn split_send_ctx<'a>(
        send_ctx: Self::SendCtx<'a>,
    ) -> (
        <(X, U) as HandoffList>::SendCtx<'a>,
        <Self::Suffix as HandoffList>::SendCtx<'a>,
    ) {
        let (x, t) = send_ctx;
        let (u, v) = <T as HandoffListSplit<U>>::split_send_ctx(t);
        ((x, u), v)
    }
}
impl<T> HandoffListSplit<()> for T
where
    T: HandoffList,
{
    type Suffix = T;

    fn split_input_port(
        input_port: Self::InputPort,
    ) -> (<() as HandoffList>::InputPort, T::InputPort) {
        ((), input_port)
    }

    #[allow(clippy::needless_lifetimes)]
    fn split_recv_ctx<'a>(
        recv_ctx: Self::RecvCtx<'a>,
    ) -> (<() as HandoffList>::RecvCtx<'a>, T::RecvCtx<'a>) {
        ((), recv_ctx)
    }

    fn split_output_port(
        output_port: Self::OutputPort,
    ) -> (<() as HandoffList>::OutputPort, T::OutputPort) {
        ((), output_port)
    }

    #[allow(clippy::needless_lifetimes)]
    fn split_send_ctx<'a>(
        send_ctx: Self::SendCtx<'a>,
    ) -> (<() as HandoffList>::SendCtx<'a>, T::SendCtx<'a>) {
        ((), send_ctx)
    }
}
