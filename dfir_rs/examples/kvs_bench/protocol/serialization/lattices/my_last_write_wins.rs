use std::cell::RefCell;
use std::rc::Rc;

use serde::de::{DeserializeSeed, Visitor};
use serde::{Serialize, Serializer};

use crate::buffer_pool::BufferPool;
use crate::protocol::serialization::lattices::with_bot::{WithBotDeserializer, WithBotWrapper};
use crate::protocol::MyLastWriteWins;

#[repr(transparent)]
pub struct MyLastWriteWinsWrapper<'a, const SIZE: usize>(pub &'a MyLastWriteWins<SIZE>);

impl<const SIZE: usize> Serialize for MyLastWriteWinsWrapper<'_, SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut struct_serializer = serializer.serialize_struct("DomPair", 2)?;

        let (key, val) = self.0.as_reveal_ref();
        struct_serializer.serialize_field("key", key)?;
        struct_serializer.serialize_field("val", &WithBotWrapper(val))?;

        struct_serializer.end()
    }
}

pub struct MyLastWriteWinsDeserializer<const SIZE: usize> {
    pub collector: Rc<RefCell<BufferPool<SIZE>>>,
}
impl<'de, const SIZE: usize> DeserializeSeed<'de> for MyLastWriteWinsDeserializer<SIZE> {
    type Value = MyLastWriteWins<SIZE>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V<const SIZE: usize> {
            pub collector: Rc<RefCell<BufferPool<SIZE>>>,
        }
        impl<'de, const SIZE: usize> Visitor<'de> for V<SIZE> {
            type Value = MyLastWriteWins<SIZE>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(std::any::type_name::<Self::Value>())
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let key = seq.next_element()?.unwrap();
                let val = seq
                    .next_element_seed(WithBotDeserializer {
                        collector: self.collector,
                    })?
                    .unwrap();

                Ok(Self::Value::new(key, val))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut key = None;
                let mut val = None;

                loop {
                    let k: Option<String> = map.next_key()?;
                    if let Some(k) = k {
                        match k.as_str() {
                            "key" => {
                                key = Some(map.next_value()?);
                            }
                            "val" => {
                                val = Some(map.next_value_seed(WithBotDeserializer {
                                    collector: self.collector.clone(),
                                })?);
                            }
                            _ => panic!(),
                        }
                    } else {
                        break;
                    }
                }

                let key = key.unwrap();
                let val = val.unwrap();

                Ok(Self::Value::new(key, val))
            }
        }

        deserializer.deserialize_struct(
            "DomPair",
            &["key", "val"],
            V {
                collector: self.collector,
            },
        )
    }
}
