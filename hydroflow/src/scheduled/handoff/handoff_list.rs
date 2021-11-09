use std::cell::RefCell;
use std::rc::Rc;

use sealed::sealed;

use crate::scheduled::ctx::{InputPort, OutputPort, RecvCtx, SendCtx};
use crate::scheduled::handoff::Handoff;
use crate::scheduled::OpId;
use crate::scheduled::util;

/**
 * A variadic list of Handoff types, represented using a lisp-style tuple structure.
 *
 * This trait is sealed and not meant to be implemented or used directly. Instead tuple lists (which already implement this trait) should be used, for example:
 * ```ignore
 * type MyHandoffList = (VecHandoff<usize>, (VecHandoff<String>, (NullHandoff, ())));
 * ```
 * The [`tl!`] (tuple list) macro simplifies usage of this kind:
 * ```ignore
 * type MyHandoffList = tl!(VecHandoff<usize>, VecHandoff<String>, NullHandoff);
 * ```
 */
#[sealed]
pub trait HandoffList {
    type RecvCtx;
    type InputPort;
    type Meta;
    fn make_input(op_id: OpId) -> (Self::RecvCtx, Self::InputPort);

    type SendCtx;
    type OutputPort;
    fn make_output(op_id: OpId) -> (Self::SendCtx, Self::OutputPort);
}
#[sealed]
impl<H, L> HandoffList for (H, L)
where
    H: 'static + Handoff,
    L: HandoffList,
{
    type RecvCtx = (RecvCtx<H>, L::RecvCtx);
    type InputPort = (InputPort<H>, L::InputPort);
    type Meta = (Rc<RefCell<H>>, L::Meta);
    fn make_input(op_id: OpId) -> (Self::RecvCtx, Self::InputPort) {
        let (send_once, once) = util::once();

        let recv = RecvCtx { once };
        let input = InputPort { op_id, once: send_once };

        let (recv_rest, input_rest) = L::make_input(op_id);

        ((recv, recv_rest), (input, input_rest))
    }

    type SendCtx = (SendCtx<H>, L::SendCtx);
    type OutputPort = (OutputPort<H>, L::OutputPort);
    fn make_output(op_id: OpId) -> (Self::SendCtx, Self::OutputPort) {
        let handoff = Rc::new(RefCell::new(H::default()));

        let send = SendCtx {
            handoff: handoff.clone(),
        };
        let output = OutputPort { op_id, handoff };

        let (send_rest, output_rest) = L::make_output(op_id);

        ((send, send_rest), (output, output_rest))
    }
}
#[sealed]
impl HandoffList for () {
    type RecvCtx = ();
    type InputPort = ();
    type Meta = ();
    fn make_input(_: OpId) -> (Self::RecvCtx, Self::InputPort) {
        ((), ())
    }

    type SendCtx = ();
    type OutputPort = ();
    fn make_output(_: OpId) -> (Self::SendCtx, Self::OutputPort) {
        ((), ())
    }
}
