use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::lang::collections::Iter;

use super::{CanReceive, Handoff, HandoffMeta};

/**
 * A [VecDeque]-based FIFO handoff.
 */
pub struct VecHandoff<T>
where
    T: 'static,
{
    pub(crate) deque: Rc<RefCell<VecDeque<T>>>,
}
impl<T> Default for VecHandoff<T>
where
    T: 'static,
{
    fn default() -> Self {
        Self {
            deque: Default::default(),
        }
    }
}
impl<T> Handoff for VecHandoff<T> {
    type Inner = VecDeque<T>;

    fn take_inner(&self) -> Self::Inner {
        self.deque.take()
    }
}

impl<T> CanReceive<Option<T>> for VecHandoff<T> {
    fn give(&self, mut item: Option<T>) -> Option<T> {
        if let Some(item) = item.take() {
            (*self.deque).borrow_mut().push_back(item)
        }
        None
    }
}
impl<T, I> CanReceive<Iter<I>> for VecHandoff<T>
where
    I: Iterator<Item = T>,
{
    fn give(&self, mut iter: Iter<I>) -> Iter<I> {
        (*self.deque).borrow_mut().extend(&mut iter.0);
        iter
    }
}
impl<T> CanReceive<VecDeque<T>> for VecHandoff<T> {
    fn give(&self, mut vec: VecDeque<T>) -> VecDeque<T> {
        (*self.deque).borrow_mut().extend(vec.drain(..));
        vec
    }
}

impl<T> HandoffMeta for VecHandoff<T> {
    fn any_ref(&self) -> &dyn Any {
        self
    }

    fn is_bottom(&self) -> bool {
        (*self.deque).borrow_mut().is_empty()
    }
}

impl<H> HandoffMeta for Rc<RefCell<H>>
where
    H: HandoffMeta,
{
    fn any_ref(&self) -> &dyn Any {
        self
    }

    fn is_bottom(&self) -> bool {
        self.borrow().is_bottom()
    }
}
