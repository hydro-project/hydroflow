use super::{AutoReturnBuffer, BufferPool};
use crate::buffer_pool::BufferType;
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer, Serialize, Serializer,
};
use std::{cell::RefCell, rc::Rc};

impl Serialize for AutoReturnBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&*self.borrow())
    }
}

pub struct AutoReturnBufferDeserializer {
    pub collector: Rc<RefCell<BufferPool>>,
}

impl<'de> DeserializeSeed<'de> for AutoReturnBufferDeserializer {
    type Value = AutoReturnBuffer;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
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
        *buff.borrow_mut() = deserializer.deserialize_bytes(BytesVisitor)?;
        Ok(buff)
    }
}

pub struct OptionalAutoReturnBufferDeserializer {
    pub collector: Rc<RefCell<BufferPool>>,
}
impl<'de> DeserializeSeed<'de> for OptionalAutoReturnBufferDeserializer {
    type Value = Option<AutoReturnBuffer>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct OptionVisitor {
            pub collector: Rc<RefCell<BufferPool>>,
        }

        impl<'de> Visitor<'de> for OptionVisitor {
            type Value = Option<AutoReturnBuffer>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(std::any::type_name::<Self::Value>())
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Some(
                    <AutoReturnBufferDeserializer as DeserializeSeed>::deserialize(
                        AutoReturnBufferDeserializer {
                            collector: self.collector,
                        },
                        deserializer,
                    )?,
                ))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_option(OptionVisitor {
            collector: self.collector,
        })
    }
}
