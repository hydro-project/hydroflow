use std::cell::RefCell;
use std::collections::VecDeque;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use futures::stream::Stream;
use pin_project::pin_project;

#[pin_project]
pub struct AsymSplit<S>
where
    S: Stream,
    S::Item: Clone,
{
    #[pin]
    stream: S,
    splits: Vec<Rc<RefCell<SideSplitState<S::Item>>>>,
}
impl<S> AsymSplit<S>
where
    S: Stream,
    S::Item: Clone,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            splits: Default::default(),
        }
    }

    pub fn add_split(&mut self) -> SideSplit<S::Item> {
        let state = Rc::new(RefCell::new(SideSplitState::default()));
        self.splits.push(state.clone());

        let split = SideSplit { state };
        split
    }
}
impl<S> Stream for AsymSplit<S>
where
    S: Stream,
    S::Item: Clone,
{
    type Item = S::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                for split in this.splits {
                    let mut split = split.borrow_mut();
                    split.buf.push_back(item.clone());
                    if let Some(waker) = &split.waker {
                        waker.wake_by_ref();
                    }
                }
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => {
                for split in this.splits {
                    let mut split = split.borrow_mut();
                    split.end = true;
                    if let Some(waker) = &split.waker {
                        waker.wake_by_ref();
                    }
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending
        }
    }
}

struct SideSplitState<T> {
    buf: VecDeque<T>,
    waker: Option<Waker>,
    end: bool,
}
impl<T> Default for SideSplitState<T> {
    fn default() -> Self {
        Self {
            buf: VecDeque::new(),
            waker: None,
            end: false,
        }
    }
}

pub struct SideSplit<T> {
    state: Rc<RefCell<SideSplitState<T>>>,
}
impl<T> Stream for SideSplit<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.state.borrow_mut();
        state.waker.replace(cx.waker().clone());
        if let Some(item) = state.buf.pop_front() {
            Poll::Ready(Some(item))
        }
        else if state.end {
            Poll::Ready(None)
        }
        else {
            Poll::Pending
        }
    }
}

#[tokio::test]
pub async fn test_asym_split_merge() {
    const BRANCH_FACTOR: usize = 10;

    use futures::StreamExt;
    use futures::future::ready;

    let stream = futures::stream::iter(0..10_000);

    seq_macro::seq!(__i in 0..10 {
        let mut splitter = AsymSplit::new(stream);
        let mut i = 0;
        let splits = [(); BRANCH_FACTOR - 1].map(|_| {
            i += 1;
            splitter.add_split().filter(move |x| ready(i == x % BRANCH_FACTOR))
        });
        let stream = super::SelectArr::new(splits);

        let splitter = splitter.filter(|x| ready(0 == x % BRANCH_FACTOR));
        let stream = futures::stream::select(splitter, stream);
    });

    let mut stream = stream;
    for i in 1.. {
        let item = stream.next().await;
        if 0 == i % 1000 {
            println!("{}: {:?}", i, item);
        }
        if item.is_none() {
            break;
        }
    }
}
