use crate::scheduled::ctx::{RecvCtx, SendCtx};
use crate::scheduled::handoff::{Handoff, HandoffList};

/**
 * Represents a compiled subgraph. Used internally by [Dataflow] to erase the input/output [Handoff] types.
 */
pub(crate) trait Subgraph {
    // TODO: pass in some scheduling info?
    fn run(&mut self);
}
/**
 * Closure-based [OpSubtree] implementation.
 */
pub(crate) struct VariadicClosureSubgraph<F, R, W>
where
    F: FnMut(&mut R::RecvCtx, &mut W::SendCtx),
    R: HandoffList,
    W: HandoffList,
{
    f: F,
    recv: R::RecvCtx,
    send: W::SendCtx,
}
impl<F, R, W> VariadicClosureSubgraph<F, R, W>
where
    F: FnMut(&mut R::RecvCtx, &mut W::SendCtx),
    R: HandoffList,
    W: HandoffList,
{
    pub fn new(f: F, recv: R::RecvCtx, send: W::SendCtx) -> Self {
        Self { f, recv, send }
    }
}
impl<F, R, W> Subgraph for VariadicClosureSubgraph<F, R, W>
where
    F: FnMut(&mut R::RecvCtx, &mut W::SendCtx),
    R: HandoffList,
    W: HandoffList,
{
    fn run(&mut self) {
        (self.f)(&mut self.recv, &mut self.send)
    }
}

pub(crate) struct NtoMClosureSubgraph<F, R, W>
where
    F: FnMut(&mut [RecvCtx<R>], &mut [SendCtx<W>]),
    R: Handoff,
    W: Handoff,
{
    f: F,
    recvs: Vec<RecvCtx<R>>,
    sends: Vec<SendCtx<W>>,
}
impl<F, R, W> NtoMClosureSubgraph<F, R, W>
where
    F: FnMut(&mut [RecvCtx<R>], &mut [SendCtx<W>]),
    R: Handoff,
    W: Handoff,
{
    pub fn new(f: F, recvs: Vec<RecvCtx<R>>, sends: Vec<SendCtx<W>>) -> Self {
        Self { f, recvs, sends }
    }
}
impl<F, R, W> Subgraph for NtoMClosureSubgraph<F, R, W>
where
    F: FnMut(&mut [RecvCtx<R>], &mut [SendCtx<W>]),
    R: Handoff,
    W: Handoff,
{
    fn run(&mut self) {
        (self.f)(&mut self.recvs, &mut self.sends)
    }
}
