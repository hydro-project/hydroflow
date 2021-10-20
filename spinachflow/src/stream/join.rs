use std::collections::HashMap;
use std::collections::VecDeque;
use std::pin::{Pin};
use std::task::{Context, Poll};

use futures::stream::Stream;
use pin_project::pin_project;

#[pin_project]
pub struct Join<A, B, K, VA, VB>
where
    A: Stream<Item = (K, VA)>,
    B: Stream<Item = (K, VB)>,
    K: std::hash::Hash + Clone + Eq,
    VA: Clone,
    VB: Clone,
{
    #[pin]
    stream_a: A,
    #[pin]
    stream_b: B,

    items_a: HashMap<K, Vec<VA>>,
    items_b: HashMap<K, Vec<VB>>,

    output_buf: VecDeque<(K, VA, VB)>,
}
impl<A, B, K, VA, VB> Join<A, B, K, VA, VB>
where
    A: Stream<Item = (K, VA)>,
    B: Stream<Item = (K, VB)>,
    K: std::hash::Hash + Clone + Eq,
    VA: Clone,
    VB: Clone,
{
    pub fn new(stream_a: A, stream_b: B) -> Self {
        let (items_a, items_b, output_buf) = Default::default();
        Self {
            stream_a,
            stream_b,

            items_a,
            items_b,
            output_buf,
        }
    }
}

impl<A, B, K, VA, VB> Stream for Join<A, B, K, VA, VB>
where
    A: Stream<Item = (K, VA)>,
    B: Stream<Item = (K, VB)>,
    K: std::hash::Hash + Clone + Eq,
    VA: Clone,
    VB: Clone,
{
    type Item = (K, VA, VB);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if let Some(item) = this.output_buf.pop_front() {
            return Poll::Ready(Some(item));
        }

        let poll = match this.stream_a.poll_next(cx) {
            Poll::Ready(Some((key, val_a))) => {
                this.items_a
                    .entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push(val_a.clone());

                if let Some(vals_b) = this.items_b.get(&key) {
                    this.output_buf.extend(vals_b.iter()
                        .cloned()
                        .map(|val_b| (key.clone(), val_a.clone(), val_b)));
                }
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        };
        if let Some(item) = this.output_buf.pop_front() {
            return Poll::Ready(Some(item));
        }

        let poll = match this.stream_b.poll_next(cx) {
            Poll::Ready(Some((key, val_b))) => {
                this.items_b
                    .entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push(val_b.clone());

                if let Some(vals_a) = this.items_a.get(&key) {
                    this.output_buf.extend(vals_a.iter()
                        .cloned()
                        .map(|val_a| (key.clone(), val_a, val_b.clone())));
                }
                Poll::Pending
            }
            Poll::Ready(None) => poll,
            Poll::Pending => Poll::Pending,
        };
        if let Some(item) = this.output_buf.pop_front() {
            return Poll::Ready(Some(item));
        }

        poll
    }
}
