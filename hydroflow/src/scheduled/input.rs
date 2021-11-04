use std::{cell::RefCell, rc::Rc};

use super::{OpId, Reactor};

pub struct Input<T> {
    reactor: Reactor,
    op_id: OpId,
    data: Rc<RefCell<Vec<T>>>,
}
impl<T> Input<T> {
    pub fn new(reactor: Reactor, op_id: OpId, data: Rc<RefCell<Vec<T>>>) -> Self {
        Input {
            reactor,
            op_id,
            data,
        }
    }

    pub fn give(&self, t: T) {
        (*self.data).borrow_mut().push(t);
    }

    pub fn flush(&self) {
        self.reactor.trigger(self.op_id).unwrap(/* TODO(justin) */);
    }

    pub fn give_vec(&self, t: &mut Vec<T>) {
        (*self.data).borrow_mut().extend(t.drain(..));
        self.flush();
    }
}
