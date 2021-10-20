use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use pin_project::pin_project;

#[pin_project]
pub struct Debug<S>
where
    S: Stream,
    S::Item: std::fmt::Debug,
{
    #[pin]
    stream: S,
    tag: String,
}
impl<S> Debug<S>
where
    S: Stream,
    S::Item: std::fmt::Debug,
{
    pub fn new(stream: S, tag: String) -> Self {
        Self { stream, tag }
    }
}

impl<S> Stream for Debug<S>
where
    S: Stream,
    S::Item: std::fmt::Debug,
{
    type Item = S::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let poll = this.stream.poll_next(cx);
        println!("{}: {:?}", this.tag, poll);
        poll
    }
}
