use std::{cell::RefCell, collections::VecDeque, marker::PhantomData, rc::Rc};

use crate::{
    progress::{Frontier, Timestamp},
    scheduled::collections::Iter,
};

use super::{CanReceive, Handoff, HandoffMeta};

pub trait Batch<D> {}

// TODO(justin): it's possible this should be an enum which is Data | Watermark.
#[derive(Debug)]
pub struct TimestampedBatch<T, D> {
    pub timestamp: T,
    pub watermark: Frontier<T>,
    pub data: Vec<D>,
}

impl<T, D> TimestampedBatch<T, D>
where
    T: Timestamp + std::fmt::Debug,
{
    pub fn new(timestamp: T, data: Vec<D>) -> Self {
        TimestampedBatch {
            timestamp,
            data,
            watermark: Frontier::new(None),
        }
    }

    pub fn close(frontier: Frontier<T>) -> Self {
        TimestampedBatch {
            timestamp: <T as Timestamp>::min(),
            data: Vec::new(),
            watermark: frontier,
        }
    }

    // TODO(justin): this is a convenience method for retaining timestamp info,
    // but it's not great for reusing memory.
    pub fn flat_map<F, U, I>(self, f: F) -> TimestampedBatch<T, U>
    where
        I: Iterator<Item = U>,
        F: Fn(D) -> I,
    {
        TimestampedBatch {
            timestamp: self.timestamp,
            watermark: self.watermark,
            data: self.data.into_iter().flat_map(f).collect(),
        }
    }

    pub fn closing(mut self, t: T) -> Self {
        self.watermark.join_in(&t);
        self
    }
}

impl<T, D> Clone for TimestampedBatch<T, D>
where
    T: Clone,
    D: Clone,
{
    fn clone(&self) -> Self {
        TimestampedBatch {
            timestamp: self.timestamp.clone(),
            data: self.data.clone(),
            watermark: self.watermark.clone(),
        }
    }
}

impl<T, D> Batch<D> for TimestampedBatch<T, D> {}

pub struct BatchedHandoff<D, B>
where
    B: Batch<D>,
{
    pub(crate) deque: Rc<RefCell<VecDeque<B>>>,
    _marker: PhantomData<D>,
}

impl<D, B> Default for BatchedHandoff<D, B>
where
    B: Batch<D>,
{
    fn default() -> Self {
        BatchedHandoff {
            deque: Rc::new(RefCell::new(VecDeque::new())),
            _marker: PhantomData,
        }
    }
}

impl<D, B> CanReceive<VecDeque<B>> for BatchedHandoff<D, B>
where
    B: Batch<D>,
{
    fn give(&self, mut batch: VecDeque<B>) -> VecDeque<B> {
        (*self.deque).borrow_mut().extend(batch.drain(..));
        batch
    }
}

impl<D, B, I> CanReceive<Iter<I>> for BatchedHandoff<D, B>
where
    I: Iterator<Item = B>,
    B: Batch<D>,
{
    fn give(&self, mut batch: Iter<I>) -> Iter<I> {
        (*self.deque).borrow_mut().extend(&mut batch.0);
        batch
    }
}

impl<D, B> CanReceive<Option<B>> for BatchedHandoff<D, B>
where
    B: Batch<D>,
{
    fn give(&self, batch: Option<B>) -> Option<B> {
        if let Some(v) = batch {
            (*self.deque).borrow_mut().push_back(v);
        }
        None
    }
}

impl<D, B> BatchedHandoff<D, B>
where
    B: Batch<D>,
{
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(B),
    {
        for batch in (*(*self.deque).borrow_mut()).drain(..) {
            f(batch)
        }
    }
}

impl<D, B> HandoffMeta for BatchedHandoff<D, B>
where
    B: 'static + Batch<D>,
    D: 'static,
{
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn is_bottom(&self) -> bool {
        (*self.deque).borrow().is_empty()
    }
}

impl<D, B> Handoff for BatchedHandoff<D, B>
where
    B: 'static + Batch<D>,
    D: 'static,
{
    type Inner = VecDeque<B>;

    fn take_inner(&self) -> Self::Inner {
        self.deque.take()
    }
}
