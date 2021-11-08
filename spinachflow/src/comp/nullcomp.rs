use std::future::Future;

use crate::op::OpDelta;

use super::{Comp, Next};

pub struct NullComp<O>
where
    O: 'static + OpDelta,
{
    op: O,
}

impl<O> NullComp<O>
where
    O: 'static + OpDelta,
{
    pub fn new(op: O) -> Self {
        Self { op }
    }
}

impl<O> Comp for NullComp<O>
where
    O: 'static + OpDelta,
{
    type Error = ();

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(_hide) = (Next { op: &self.op }).await {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}
