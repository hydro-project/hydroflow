use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;

use crate::op::OpDelta;
use crate::hide::{Hide, Delta};

struct Next<'s, O: OpDelta> {
    op: &'s O,
}

impl<O: OpDelta> Future for Next<'_, O> {
    type Output = Option<Hide<Delta, O::LatRepr>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.op.poll_delta(ctx)
    }
}

mod comptrait;
pub use comptrait::*;

mod nullcomp;
pub use nullcomp::*;

mod debugcomp;
pub use debugcomp::*;

mod tcpcomp;
pub use tcpcomp::*;

mod tcpservercomp;
pub use tcpservercomp::*;

mod dynsplitcomp;
pub use dynsplitcomp::*;
