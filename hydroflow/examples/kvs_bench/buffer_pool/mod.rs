mod serialization;

pub use serialization::AutoReturnBufferDeserializer;

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

pub type BufferType = [u8; 1024];

#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<Rc<RefCell<BufferType>>>,
}

#[derive(Clone, Debug)]
pub struct AutoReturnBufferInner {
    pub collector: Weak<RefCell<BufferPool>>,
    pub inner: Rc<RefCell<BufferType>>,
}

#[derive(Clone, Debug, Default)]
pub struct AutoReturnBuffer {
    pub inner: Option<AutoReturnBufferInner>,
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
                inner: Some(AutoReturnBufferInner {
                    collector: Rc::downgrade(pool),
                    inner: buffer,
                }),
            }
        } else {
            AutoReturnBuffer {
                inner: Some(AutoReturnBufferInner {
                    collector: Rc::downgrade(pool),
                    inner: Rc::new(RefCell::new([0; 1024])),
                }),
            }
        }
    }
}

impl Drop for AutoReturnBuffer {
    fn drop(&mut self) {
        if let Some(inner) = &self.inner {
            if Rc::strong_count(&self.inner.as_ref().unwrap().inner) == 1 {
                // This is the last one, give the buffer back to the collection.
                inner
                    .collector
                    .upgrade()
                    .unwrap()
                    .borrow_mut()
                    .buffers
                    .push(inner.inner.clone());
            }
        }
    }
}

impl AutoReturnBuffer {
    pub fn _borrow_mut(&self) -> Option<RefMut<BufferType>> {
        self.inner.as_ref().map(|x| x.inner.borrow_mut())
    }

    pub fn borrow(&self) -> Option<Ref<BufferType>> {
        self.inner.as_ref().map(|x| x.inner.borrow())
    }
}

// This is a lie, if these buffers are shared across threads it will explode.
// They are marked as send and sync so they can be used in hydroflow channels in a single threaded transducer.
// Otherwise we would need to make our own channel type.
unsafe impl Send for AutoReturnBuffer {}
unsafe impl Sync for AutoReturnBuffer {}

impl PartialEq for AutoReturnBuffer {
    fn eq(&self, other: &AutoReturnBuffer) -> bool {
        match (&self.inner, &other.inner) {
            (Some(x), Some(y)) => x.inner == y.inner,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}
impl Eq for AutoReturnBuffer {}
