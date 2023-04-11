use super::KvsRequest;
use crate::{buffer_pool::BufferPool, protocol::serialization::KvsRequestDeserializer};
use bincode::options;
use serde::{de::DeserializeSeed, Serialize};
use serde_json::de::StrRead;
use std::{io::Cursor, rc::Rc};

#[test]
fn test_bincode() {
    let buffer_pool = BufferPool::create_buffer_pool();

    let req = KvsRequest::Gossip {
        key: 7,
        reg: (49, BufferPool::get_from_buffer_pool(&buffer_pool)),
    };

    let mut serialized = Vec::new();

    let mut serializer = bincode::Serializer::new(Cursor::new(&mut serialized), options());
    Serialize::serialize(&req, &mut serializer).unwrap();

    println!("{:?}", serialized);

    let mut deserializer = bincode::Deserializer::from_slice(&serialized, options());
    let req2 = KvsRequestDeserializer {
        collector: Rc::downgrade(&buffer_pool),
    }
    .deserialize(&mut deserializer)
    .unwrap();

    assert_eq!(req, req2);
}

#[test]
fn test_json() {
    let buffer_pool = BufferPool::create_buffer_pool();

    let req = KvsRequest::Gossip {
        key: 7,
        reg: (49, BufferPool::get_from_buffer_pool(&buffer_pool)),
    };

    let serialized = serde_json::to_string(&req).unwrap();

    let mut deserializer = serde_json::Deserializer::new(StrRead::new(&serialized));
    let req2 = KvsRequestDeserializer {
        collector: Rc::downgrade(&buffer_pool),
    }
    .deserialize(&mut deserializer)
    .unwrap();

    assert_eq!(req, req2);
}
