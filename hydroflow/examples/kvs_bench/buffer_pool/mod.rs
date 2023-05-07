mod serialization;

pub use serialization::AutoReturnBufferDeserializer;
pub use serialization::OptionalAutoReturnBufferDeserializer;

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

pub const BUFFER_SIZE: usize = 1024;
pub type BufferType = [u8; BUFFER_SIZE];

/// It is slow to allocate and, in particular, free blocks of memory.
/// If we know ahead of time what size buffers we will need then we can pre-allocate a bunch of them and
/// reuse those allocations.
/// To comply with rust's type system, these buffers have shared owernship and interior mutability, so Rc<RefCell>
/// Eventually these buffers can be made more dynamic by using scatter-gather I/O and a buffer-of-buffers pattern.
/// We are not there yet and so this is a simple approximation of what it will eventually look like.
#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<Rc<RefCell<BufferType>>>,
}

/// This buffer will be returned to it's owning buffer pool when it is dropped.
/// In order to return it, it has to know where it is being returned to, this is the [collector] member.
/// The collector pointer is weak because otherwise there would be a cycle between BufferPool and AutoReturnBufferInner
/// and an AutoReturnBufferInner does not logically have any kind of ownership over the BufferPool shared pool.
#[derive(Clone, Debug)]
pub struct AutoReturnBuffer {
    pub collector: Weak<RefCell<BufferPool>>,
    pub inner: Rc<RefCell<BufferType>>,
}

impl BufferPool {
    pub fn create_buffer_pool() -> Rc<RefCell<BufferPool>> {
        Rc::new(RefCell::new(BufferPool {
            buffers: Vec::new(),
        }))
    }

    pub fn get_from_buffer_pool(pool: &Rc<RefCell<BufferPool>>) -> AutoReturnBuffer {
        let buffer = pool.borrow_mut().buffers.pop();

        if let Some(buffer) = buffer {
            AutoReturnBuffer {
                collector: Rc::downgrade(pool),
                inner: buffer,
            }
        } else {
            AutoReturnBuffer {
                collector: Rc::downgrade(pool),
                inner: Rc::new(RefCell::new([0; BUFFER_SIZE])),
            }
        }
    }
}

impl Drop for AutoReturnBuffer {
    fn drop(&mut self) {
        if Rc::strong_count(&self.inner) == 1 {
            // This is the last one, give the buffer back to the collection.
            if let Some(pool) = self.collector.upgrade() {
                pool.borrow_mut().buffers.push(self.inner.clone());
            }

            // If the upgrade fails then the buffer pool has been freed and so there
            // is nothing to return this buffer to, just drop it instead.
        }
    }
}

impl AutoReturnBuffer {
    pub fn borrow_mut(&self) -> RefMut<BufferType> {
        self.inner.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<BufferType> {
        self.inner.borrow()
    }
}
