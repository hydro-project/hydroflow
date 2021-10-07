use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::stream::Stream;
use std::task::{Context, Poll, Waker};

pub struct Splitter<S: Stream> {
    stream: Rc<RefCell<Pin<Box<S>>>>,
    splits: Rc<Vec<Rc<RefCell<SplitData<S::Item>>>>>,
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
            splits: Rc::new(Vec::new()),
        }
    }
    pub fn add_split(&mut self) -> Split<S> {
        let data = Rc::new(RefCell::new(SplitData::default()));
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

        // Get our index.
        let index = self.splitter.splits.iter()
            .enumerate()
            .find(|(_, split_other)| Rc::ptr_eq(&self.data, split_other))
            .unwrap().0;

        // Iterate in circular order, so each successive split checks the next split.
        let (splits_before, splits_after) = self.splitter.splits.split_at(index);
        let splits_after = &splits_after[1..]; // Skip self.

        // Check if other splits are ready to receive a value.
        for split in splits_after.iter().chain(splits_before.iter()) {
            let split = split.borrow();
            if let Some(_) = split.item {
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

                    if let Some(waker) = split.waker.take() {
                        waker.wake();
                    }
                }
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}