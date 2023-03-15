use std::{
    array::IntoIter,
    borrow::Borrow,
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<Rc<[u8; 1024]>>,
}

// let buffer_pool = Rc::new(RefCell::new(BufferPool::default()));

#[derive(Clone, Debug)]
struct AutoReturnBufferInner {
    collector: Weak<RefCell<BufferPool>>,
    inner: Rc<[u8; 1024]>,
}

#[derive(Clone, Debug)]
pub struct AutoReturnBuffer {
    inner: Option<AutoReturnBufferInner>,
}

impl BufferPool {
    /// Once we have any-self-types then this can be a bit neater with self: Rc<RefCell<Self>>
    pub fn create_buffer_pool() -> Rc<RefCell<BufferPool>> {
        Rc::new(RefCell::new(BufferPool {
            buffers: Vec::new(),
        }))
    }

    /// Once we have any-self-types then this can be a bit neater with self: Rc<RefCell<Self>>
    pub fn get_from_buffer_pool(pool: Rc<RefCell<BufferPool>>) -> AutoReturnBuffer {
        let x = pool.borrow_mut().buffers.pop();

        if let Some(buffer) = x {
            AutoReturnBuffer {
                inner: Some(AutoReturnBufferInner {
                    collector: Rc::downgrade(&pool),
                    inner: buffer,
                }),
            }
        } else {
            AutoReturnBuffer {
                inner: Some(AutoReturnBufferInner {
                    collector: Rc::downgrade(&pool),
                    inner: Rc::new([0; 1024]),
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

/// This is a lie, if these buffers are shared across threads it will explode.
unsafe impl Send for AutoReturnBuffer {}
unsafe impl Sync for AutoReturnBuffer {}

impl Default for AutoReturnBuffer {
    fn default() -> Self {
        Self { inner: None }
    }
}

impl Deref for AutoReturnBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        if let Some(inner) = &self.inner {
            inner.inner.as_slice()
        } else {
            &[]
        }
    }
}

impl AsRef<[u8]> for AutoReturnBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        if let Some(inner) = &self.inner {
            inner.inner.as_slice()
        } else {
            &[]
        }
    }
}

impl Borrow<[u8]> for AutoReturnBuffer {
    fn borrow(&self) -> &[u8] {
        if let Some(inner) = &self.inner {
            inner.inner.as_slice()
        } else {
            &[]
        }
    }
}

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

impl IntoIterator for AutoReturnBuffer {
    type Item = u8;
    type IntoIter = IntoIter<u8, 1024>;

    fn into_iter(self) -> Self::IntoIter {
        if let Some(inner) = &self.inner {
            inner.inner.into_iter()
        } else {
            panic!()
        }
    }
}

impl<'a> IntoIterator for &'a AutoReturnBuffer {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        if let Some(inner) = &self.inner {
            inner.inner.as_slice().into_iter()
        } else {
            panic!()
        }
    }
}

// impl PartialOrd for Bytes {
//     fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
//         self.as_slice().partial_cmp(other.as_slice())
//     }
// }

// impl Ord for Bytes {
//     fn cmp(&self, other: &Bytes) -> cmp::Ordering {
//         self.as_slice().cmp(other.as_slice())
//     }
// }
