use super::magic_buffer::MagicBuffer;
use crate::{
    buffer_pool::BufferPool,
    protocol::{serialization::KvsRequestDeserializer, KvsRequest},
};
use bincode::options;
use serde::{de::DeserializeSeed, Serialize};
use std::{cell::RefCell, rc::Rc};

fn assert_eq_req(r1: &KvsRequest, r2: &KvsRequest) {
    match (r1, r2) {
        (
            KvsRequest::Gossip {
                key: key1,
                reg: reg1,
            },
            KvsRequest::Gossip {
                key: key2,
                reg: reg2,
            },
        ) => {
            assert_eq!(key1, key2);
            assert_eq!(reg1.key, reg2.key);
            assert_eq!(
                *reg1.val.0.as_ref().unwrap().0.inner.borrow(),
                *reg2.val.0.as_ref().unwrap().0.inner.borrow()
            );
        }
        (KvsRequest::Delete { key: key1 }, KvsRequest::Delete { key: key2 }) => {
            assert_eq!(key1, key2);
        }
        (KvsRequest::Get { key: key1 }, KvsRequest::Get { key: key2 }) => {
            assert_eq!(key1, key2);
        }
        (
            KvsRequest::Put {
                key: key1,
                value: value1,
            },
            KvsRequest::Put {
                key: key2,
                value: value2,
            },
        ) => {
            assert_eq!(key1, key2);
            assert_eq!(*value1.inner.borrow(), *value2.inner.borrow());
        }
        _ => panic!(),
    }
}

fn serialize_deserialize<'de, S, D>(
    req: &KvsRequest,
    buffer_pool: &Rc<RefCell<BufferPool>>,
    mut serializer: S,
    mut deserializer: D,
) -> KvsRequest
where
    for<'a> &'a mut S: serde::Serializer,
    for<'b> &'b mut D: serde::Deserializer<'de>,
{
    Serialize::serialize(&req, &mut serializer).unwrap();

    let req2 = {
        KvsRequestDeserializer {
            collector: Rc::clone(&buffer_pool),
        }
        .deserialize(&mut deserializer)
        .unwrap()
    };

    req2
}

pub fn check_all(buffer_pool: &Rc<RefCell<BufferPool>>, req: &KvsRequest) {
    {
        let buffer = MagicBuffer::default();

        let req2 = serialize_deserialize(
            &req,
            &buffer_pool,
            bincode::Serializer::new(buffer.clone(), options()),
            bincode::Deserializer::with_reader(buffer.clone(), options()),
        );

        assert_eq_req(&req, &req2);
    }

    {
        let buffer = MagicBuffer::default();

        let req2 = serialize_deserialize(
            &req,
            &buffer_pool,
            serde_json::Serializer::new(buffer.clone()),
            serde_json::Deserializer::from_reader(buffer.clone()),
        );

        assert_eq_req(&req, &req2);
    }
}
