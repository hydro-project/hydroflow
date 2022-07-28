use std::marker::PhantomData;

use pusherator::Pusherator;

use crate::scheduled::handoff::{CanReceive, Handoff};
use crate::scheduled::port::SendCtx;

pub struct PushHandoff<'a, H, T>
where
    H: Handoff + CanReceive<T>,
{
    send_ctx: &'a SendCtx<H>,
    _phantom: PhantomData<fn(T)>,
}
impl<'a, H, T> PushHandoff<'a, H, T>
where
    H: Handoff + CanReceive<T>,
{
    pub fn new(send_ctx: &'a SendCtx<H>) -> Self {
        Self {
            send_ctx,
            _phantom: PhantomData,
        }
    }
}
impl<'a, H, T> Pusherator for PushHandoff<'a, H, T>
where
    H: Handoff + CanReceive<T>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        self.send_ctx.give(item);
    }
}
