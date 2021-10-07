use std::pin::Pin;
use std::stream::Stream;
use std::task::{Context, Poll};

use pin_project::pin_project;

#[pin_project]
pub struct Merge<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    #[pin]
    stream_a: A,
    #[pin]
    stream_b: B,
}

impl<A, B> Merge<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    pub fn new(stream_a: A, stream_b: B) -> Self {
        Self { stream_a, stream_b }
    }
}

impl<A, B> Stream for Merge<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    type Item = A::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let not_ready = match this.stream_a.poll_next(cx) {
            Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        };
        match this.stream_b.poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => not_ready,
            Poll::Pending => Poll::Pending,
        }
    }
}
