use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use futures::stream::Stream;

pub struct Splitter<S: Stream> {
    stream: Rc<RefCell<Pin<Box<S>>>>,
    splits: Rc<RefCell<Vec<Rc<RefCell<SplitData<S::Item>>>>>>,
}
impl<S: Stream> Clone for Splitter<S> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            splits: self.splits.clone(),
        }
    }
}
impl<S: Stream> Splitter<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream: Rc::new(RefCell::new(Box::pin(stream))),
            splits: Rc::new(RefCell::new(Vec::new())),
        }
    }
    pub fn add_split(&self) -> Split<S> {
        let data = Rc::new(RefCell::new(SplitData::default()));
        self.splits.borrow_mut().push(data.clone());
        Split {
            splitter: self.clone(),
            data,
        }
    }
}

pub struct SplitData<T> {
    item: Option<T>,
    waker: Option<Waker>,
}
impl<T> Default for SplitData<T> {
    fn default() -> Self {
        Self {
            item: Default::default(),
            waker: Default::default(),
        }
    }
}

pub struct Split<S: Stream> {
    splitter: Splitter<S>,
    data: Rc<RefCell<SplitData<S::Item>>>,
}
impl<S: Stream> Stream for Split<S>
where
    S::Item: Clone,
{
    type Item = S::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut data = self.data.borrow_mut();
        match data.item.take() {
            Some(item) => {
                return Poll::Ready(Some(item));
            }
            None => {
                data.waker.replace(cx.waker().clone());
            }
        }
        let splits = self.splitter.splits.borrow();

        // Get our index.
        let index = splits
            .iter()
            .enumerate()
            .find(|(_, split_other)| Rc::ptr_eq(&self.data, split_other))
            .unwrap()
            .0;

        // Iterate in circular order, so each successive split checks the next split.
        let (splits_before, splits_after) = splits.split_at(index);
        let splits_after = &splits_after[1..]; // Skip self.

        // Check if other splits are ready to receive a value.
        for split in splits_after.iter().chain(splits_before.iter()) {
            let split = split.borrow();
            if split.item.is_some() {
                // If any split has it's value filled, wake it up and return pending.
                if let Some(waker) = &split.waker {
                    waker.wake_by_ref();
                }
                return Poll::Pending;
            }
        }

        // Poll upstream.
        let mut stream = self.splitter.stream.borrow_mut();
        match stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(item)) => {
                for split in splits_after.iter().chain(splits_before.iter()) {
                    let mut split = split.borrow_mut();
                    let old_item = split.item.replace(item.clone());
                    assert!(old_item.is_none());

                    if let Some(waker) = &split.waker {
                        waker.wake_by_ref();
                    }
                }
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[tokio::test]
pub async fn test_split_merge() {
    const BRANCH_FACTOR: usize = 10;

    use futures::future::ready;
    use futures::StreamExt;

    let stream = futures::stream::iter(0..10_000);

    seq_macro::seq!(__i in 0..20 {
        let splitter = Splitter::new(stream);
        let mut i = 0;
        let splits = [(); BRANCH_FACTOR].map(|_| {
            let r = i;
            i += 1;
            splitter.add_split().filter(move |x| ready(r == x % BRANCH_FACTOR))
        });
        let stream = super::SelectArr::new(splits);
        let stream: Pin<Box<dyn Stream<Item = usize>>> = Box::pin(stream);
    });

    let mut stream = stream;
    for _i in 1.. {
        let item = stream.next().await;
        // if 0 == i % 1000 {
        //     println!("{}: {:?}", i, item);
        // }
        if item.is_none() {
            break;
        }
    }
}
