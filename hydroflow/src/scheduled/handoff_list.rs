use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use ref_cast::RefCast;
use sealed::sealed;

use crate::scheduled::ctx::{InputPort, OutputPort, RecvCtx, SendCtx};
use crate::scheduled::handoff::Handoff;

use super::graph::HandoffData;
use super::{HandoffId, SubgraphId};

/**
 * A variadic list of Handoff types, represented using a lisp-style tuple structure.
 *
 * This trait is sealed and not meant to be implemented or used directly. Instead tuple lists (which already implement this trait) should be used, for example:
 * ```ignore
 * type MyHandoffList = (VecHandoff<usize>, (VecHandoff<String>, (TeeingHandoff<u32>, ())));
 * ```
 * The [`tl!`] (tuple list) macro simplifies usage of this kind:
 * ```ignore
 * type MyHandoffList = tl!(VecHandoff<usize>, VecHandoff<String>, TeeingHandoff<u32>);
 * ```
 */
#[sealed]
pub trait HandoffList {
    type InputHid;
    type InputPort;
    type RecvCtx<'a>;
    fn make_input(sg_id: SubgraphId) -> (Self::InputHid, Self::InputPort);
    fn make_recv<'a>(handoffs: &'a [HandoffData], input_hids: &Self::InputHid)
        -> Self::RecvCtx<'a>;

    type OutputHid;
    type OutputPort;
    type SendCtx<'a>;
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
