use std::future::Future;

use crate::op::OpDelta;

use super::{Comp, Next};

pub struct NullComp<O: OpDelta> {
    op: O,
}

impl<O: OpDelta> NullComp<O> {
    pub fn new(op: O) -> Self {
        Self { op }
    }
}

impl<O: OpDelta> Comp for NullComp<O> {
    type Error = ();

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            if let Some(_hide) = (Next { op: &self.op }).await {
                Ok(())
            }
            else {
                Err(())
            }
        }
    }
}
