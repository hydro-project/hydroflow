mod magic_buffer;
mod util;

use self::util::check_all;
use super::KvsRequest;
use crate::{buffer_pool::BufferPool, protocol::MyLastWriteWins};
use lattices::{bottom::Bottom, fake::Fake, ord::Max};

#[test]
fn test_gossip() {
    let buffer_pool = BufferPool::create_buffer_pool();

    let req = {
        let buffer = BufferPool::get_from_buffer_pool(&buffer_pool);
        buffer.borrow_mut()[0] = 117;
        let reg = MyLastWriteWins::new(Max::new(49), Bottom::new(Fake::new(buffer)));

        KvsRequest::Gossip { key: 7, reg }
    };

    check_all(&buffer_pool, &req);
}

#[test]
fn test_delete() {
    let buffer_pool = BufferPool::create_buffer_pool();

    check_all(&buffer_pool, &KvsRequest::Delete { key: 7 });
}

#[test]
fn test_get() {
    let buffer_pool = BufferPool::create_buffer_pool();

    check_all(&buffer_pool, &KvsRequest::Get { key: 7 });
}

#[test]
fn test_put() {
    let buffer_pool = BufferPool::create_buffer_pool();

    let req = {
        let value = BufferPool::get_from_buffer_pool(&buffer_pool);
        value.borrow_mut()[0] = 117;

        KvsRequest::Put { key: 7, value }
    };

    check_all(&buffer_pool, &req);
}
