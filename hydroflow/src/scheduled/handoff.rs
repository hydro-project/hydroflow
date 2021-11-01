use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::scheduled::collections::Iter;

pub trait TryCanReceive<T> {
    fn try_give(&mut self, item: T) -> Result<T, T>;
}
pub trait CanReceive<T> {
    fn give(&mut self, item: T) -> T;
}

pub trait Handoff: Default + HandoffMeta {
    type Inner;

    fn take_inner(&mut self) -> Self::Inner;

    fn give<T>(&mut self, item: T) -> T
    where
        Self: CanReceive<T>,
    {
        <Self as CanReceive<T>>::give(self, item)
    }

    fn try_give<T>(&mut self, item: T) -> Result<T, T>
    where
        Self: TryCanReceive<T>,
    {
        <Self as TryCanReceive<T>>::try_give(self, item)
    }
}

#[derive(Default)]
pub struct NullHandoff;
impl Handoff for NullHandoff {
    type Inner = ();
    fn take_inner(&mut self) -> Self::Inner {}
}

/**
 * A [VecDeque]-based FIFO handoff.
 */
pub struct VecHandoff<T> {
    pub(crate) deque: Rc<RefCell<VecDeque<T>>>,
}
impl<T> Default for VecHandoff<T> {
    fn default() -> Self {
        Self {
            deque: Default::default(),
        }
    }
}
impl<T> Handoff for VecHandoff<T> {
    type Inner = VecDeque<T>;

    fn take_inner(&mut self) -> Self::Inner {
        self.deque.take()
    }
}

impl<T> CanReceive<Option<T>> for VecHandoff<T> {
    fn give(&mut self, mut item: Option<T>) -> Option<T> {
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
    fn give(&mut self, mut iter: Iter<I>) -> Iter<I> {
        (*self.deque).borrow_mut().extend(&mut iter.0);
        iter
    }
}
impl<T> CanReceive<VecDeque<T>> for VecHandoff<T> {
    fn give(&mut self, mut vec: VecDeque<T>) -> VecDeque<T> {
        (*self.deque).borrow_mut().extend(vec.drain(..));
        vec
    }
}

/**
 * A handle onto the metadata part of a [Handoff], with no element type.
 */
pub trait HandoffMeta {
    // TODO(justin): more fine-grained info here.
    fn is_bottom(&self) -> bool;
}

impl HandoffMeta for NullHandoff {
    fn is_bottom(&self) -> bool {
        true
    }
}

impl<T> HandoffMeta for VecHandoff<T> {
    fn is_bottom(&self) -> bool {
        (*self.deque).borrow_mut().is_empty()
    }
}

impl<H> HandoffMeta for Rc<RefCell<H>>
where
    H: HandoffMeta,
{
    fn is_bottom(&self) -> bool {
        self.borrow().is_bottom()
    }
}

// TODO(justin): all this needs optimization.
struct ReaderHandoff<T> {
    contents: Vec<T>,
}

impl<T> Default for ReaderHandoff<T> {
    fn default() -> Self {
        Self {
            contents: Default::default(),
        }
    }
}

struct TeeingHandoffInternal<T> {
    readers: Vec<ReaderHandoff<T>>,
}

// A [Handoff] which is part of a "family" of handoffs. Writing to this handoff
// will write to every reader. New readers can be created by calling `tee`.
#[derive(Clone)]
pub struct TeeingHandoff<T> {
    read_from: usize,
    internal: Rc<RefCell<TeeingHandoffInternal<T>>>,
}

impl<T> Default for TeeingHandoff<T> {
    fn default() -> Self {
        TeeingHandoff {
            read_from: 0,
            internal: Rc::new(RefCell::new(TeeingHandoffInternal {
                readers: vec![Default::default()],
            })),
        }
    }
}

impl<T: Clone> TeeingHandoff<T> {
    pub fn tee(&self) -> Self {
        let id = (*self.internal).borrow().readers.len();
        (*self.internal)
            .borrow_mut()
            .readers
            .push(ReaderHandoff::default());
        Self {
            read_from: id,
            internal: self.internal.clone(),
        }
    }
}

impl<T> HandoffMeta for TeeingHandoff<T> {
    fn is_bottom(&self) -> bool {
        true
    }
}

impl<T> Handoff for TeeingHandoff<T> {
    type Inner = Vec<T>;
    fn take_inner(&mut self) -> Self::Inner {
        std::mem::take(&mut (*self.internal).borrow_mut().readers[self.read_from].contents)
    }
}

impl<T> CanReceive<Vec<T>> for TeeingHandoff<T>
where
    T: Clone,
{
    fn give(&mut self, vec: Vec<T>) -> Vec<T> {
        let readers = &mut (*self.internal).borrow_mut().readers;
        for i in 0..(readers.len() - 1) {
            readers[i].contents.extend_from_slice(&vec);
        }
        let last = readers.len() - 1;
        readers[last].contents.extend(vec);
        Vec::new()
    }
}
