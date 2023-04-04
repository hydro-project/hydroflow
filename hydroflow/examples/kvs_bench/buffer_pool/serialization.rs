use super::{AutoReturnBuffer, BufferPool};
use crate::buffer_pool::{AutoReturnBufferInner, BufferType};
use serde::{
    de::{DeserializeSeed, Visitor},
    Serialize, Serializer,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

impl Serialize for AutoReturnBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&*self.borrow().unwrap())
    }
}

pub struct AutoReturnBufferDeserializer {
    pub collector: Weak<RefCell<BufferPool>>,
}

impl<'de> DeserializeSeed<'de> for AutoReturnBufferDeserializer {
    type Value = AutoReturnBuffer;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;
        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = BufferType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("[u8; _]")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.try_into().unwrap()) // TODO: proper error mapping
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(e) = seq.next_element()? {
                    vec.push(e);
                }

                Ok(vec.as_slice().try_into().unwrap()) // TODO: proper error mapping
            }
        }

        Ok(AutoReturnBuffer {
            inner: Some(AutoReturnBufferInner {
                collector: self.collector,
                inner: Rc::new(RefCell::new(deserializer.deserialize_bytes(BytesVisitor)?)),
            }),
        })
    }
}
