use super::{AutoReturnBuffer, BufferPool};
use crate::buffer_pool::BufferType;
use serde::{
    de::{DeserializeSeed, Visitor},
    Serialize, Serializer,
};
use std::{cell::RefCell, rc::Rc};

impl Serialize for AutoReturnBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&*self.borrow().unwrap())
    }
}

pub struct AutoReturnBufferDeserializer {
    pub collector: Rc<RefCell<BufferPool>>,
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
                formatter.write_str(std::any::type_name::<Self::Value>())
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.try_into().map_err(E::custom)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(e) = seq.next_element()? {
                    vec.push(e);
                }

                vec.as_slice().try_into().map_err(serde::de::Error::custom)
            }
        }

        let buff = BufferPool::get_from_buffer_pool(&self.collector);

        {
            let mut borrow = buff.borrow_mut().unwrap();
            *borrow = deserializer.deserialize_bytes(BytesVisitor)?;
        }

        Ok(buff)
    }
}
