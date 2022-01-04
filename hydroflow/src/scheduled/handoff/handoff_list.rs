use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use ref_cast::RefCast;
use sealed::sealed;

use crate::scheduled::ctx::{InputPort, OutputPort, RecvCtx, SendCtx};
use crate::scheduled::graph::HandoffData;
use crate::scheduled::type_list::TypeList;
use crate::scheduled::{HandoffId, SubgraphId};

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
    type InputHid: TypeList;
    type InputPort: TypeList;
    type RecvCtx<'a>: TypeList;
    fn make_input(sg_id: SubgraphId) -> (Self::InputHid, Self::InputPort);
    fn make_recv<'a>(handoffs: &'a [HandoffData], input_hids: &Self::InputHid)
        -> Self::RecvCtx<'a>;

    type OutputHid: TypeList;
    type OutputPort: TypeList;
    type SendCtx<'a>: TypeList;
    fn make_output(sg_id: SubgraphId) -> (Self::OutputHid, Self::OutputPort);
    fn make_send<'a>(
        handoffs: &'a [HandoffData],
        output_hids: &Self::OutputHid,
    ) -> Self::SendCtx<'a>;
}
#[sealed]
impl<H, L> HandoffList for (H, L)
where
    H: 'static + Handoff,
    L: HandoffList,
{
    type InputHid = (Rc<Cell<Option<HandoffId>>>, L::InputHid);
    type InputPort = (InputPort<H>, L::InputPort);
    type RecvCtx<'a> = (&'a RecvCtx<H>, L::RecvCtx<'a>);
    fn make_input(sg_id: SubgraphId) -> (Self::InputHid, Self::InputPort) {
        let hid = <Rc<Cell<Option<HandoffId>>>>::default();
        let input = InputPort {
            sg_id,
            handoff_id: hid.clone(),
            _phantom: PhantomData,
        };

        let (hid_rest, input_rest) = L::make_input(sg_id);

        ((hid, hid_rest), (input, input_rest))
    }
    fn make_recv<'a>(
        handoffs: &'a [HandoffData],
        input_hids: &Self::InputHid,
    ) -> Self::RecvCtx<'a> {
        let (hid, hid_rest) = input_hids;
        let hid = hid.get().expect("Attempted to use unattached handoff.");
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

    type OutputHid = (Rc<Cell<Option<HandoffId>>>, L::OutputHid);
    type OutputPort = (OutputPort<H>, L::OutputPort);
    type SendCtx<'a> = (&'a SendCtx<H>, L::SendCtx<'a>);
    fn make_output(sg_id: SubgraphId) -> (Self::OutputHid, Self::OutputPort) {
        let hid = <Rc<Cell<Option<HandoffId>>>>::default();
        let output = OutputPort {
            sg_id,
            handoff_id: hid.clone(),
            _phantom: PhantomData,
        };

        let (hid_rest, output_rest) = L::make_output(sg_id);

        ((hid, hid_rest), (output, output_rest))
    }
    fn make_send<'a>(
        handoffs: &'a [HandoffData],
        output_hids: &Self::OutputHid,
    ) -> Self::SendCtx<'a> {
        let (hid, hid_rest) = output_hids;
        let hid = hid.get().expect("Attempted to use unattached handoff.");
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
    type InputHid = ();
    type InputPort = ();
    type RecvCtx<'a> = ();
    fn make_input(_: SubgraphId) -> (Self::InputHid, Self::InputPort) {
        ((), ())
    }
    fn make_recv<'a>(
        _handoffs: &'a [HandoffData],
        _input_hids: &Self::InputHid,
    ) -> Self::RecvCtx<'a> {
    }

    type OutputHid = ();
    type OutputPort = ();
    type SendCtx<'a> = ();
    fn make_output(_: SubgraphId) -> (Self::OutputHid, Self::OutputPort) {
        ((), ())
    }
    fn make_send<'a>(
        _handoffs: &'a [HandoffData],
        _output_hids: &Self::OutputHid,
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
