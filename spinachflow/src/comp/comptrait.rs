use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::op::Op;

pub struct CompRunFuture<'s, C: Comp + ?Sized> {
    comp: &'s C,
    future: Pin<Box<C::TickFuture<'s>>>,
}

impl<'s, C: Comp + ?Sized> Future for CompRunFuture<'s, C> {
    type Output = Result<!, C::Error>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.future.as_mut().poll(ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Ready(Ok(_)) => {
                let this = self.get_mut();
                this.future = Box::pin(this.comp.tick());
                Pin::new(this).poll(ctx)
            }
        }
    }
}

pub trait Comp {
    type Error: std::fmt::Debug;

    type TickFuture<'s>: Future<Output = Result<(), Self::Error>>
    where
        Self: 's;
    fn tick(&self) -> Self::TickFuture<'_>;
}

pub trait CompExt: Comp {
    fn run(&self) -> CompRunFuture<'_, Self> {
        CompRunFuture {
            comp: self,
            future: Box::pin(self.tick()),
        }
    }
}
impl<C: Comp> CompExt for C {}

pub trait CompConnector<O: Op> {
    type Comp: Comp;

    #[must_use]
    fn connect(&self, op: O) -> Self::Comp;
}
