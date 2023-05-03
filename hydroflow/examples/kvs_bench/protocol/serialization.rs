use super::KvsRequest;
use crate::buffer_pool::{AutoReturnBuffer, AutoReturnBufferDeserializer, BufferPool};
use hydroflow::lang::lattice2::{dom_pair::DomPair, fake::Fake, ord::Max};
use serde::{
    de::{DeserializeSeed, SeqAccess, VariantAccess, Visitor},
    ser::SerializeStructVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{cell::RefCell, rc::Rc};

impl Serialize for KvsRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            KvsRequest::Put { key, value } => {
                let mut s = serializer.serialize_struct_variant("KvsRequest", 0, "Put", 2)?;
                s.serialize_field("key", key)?;
                s.serialize_field("value", value)?;
                s.end()
            }
            KvsRequest::_Get { key } => {
                let mut s = serializer.serialize_struct_variant("KvsRequest", 1, "Get", 1)?;
                s.serialize_field("key", key)?;
                s.end()
            }
            KvsRequest::Gossip { key, reg } => {
                let mut s = serializer.serialize_struct_variant("KvsRequest", 2, "Gossip", 3)?;
                s.serialize_field("key", key)?;
                s.serialize_field("marker", &reg.key.0)?;
                s.serialize_field("buffer", &reg.val.0)?;
                s.end()
            }
        }
    }
}

pub struct KvsRequestDeserializer {
    pub collector: Rc<RefCell<BufferPool>>,
}

impl<'de> DeserializeSeed<'de> for KvsRequestDeserializer {
    type Value = KvsRequest;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct KvsRequestVisitor {
            collector: Rc<RefCell<BufferPool>>,
        }
        impl<'de> Visitor<'de> for KvsRequestVisitor {
            type Value = KvsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("KvsRequest enum")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                enum KvsRequestField {
                    Put,
                    Get,
                    Gossip,
                }
                struct KVSRequest2FieldVisitor;
                impl<'de> Visitor<'de> for KVSRequest2FieldVisitor {
                    type Value = KvsRequestField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("field identifier")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            0 => Result::Ok(KvsRequestField::Put),
                            1 => Result::Ok(KvsRequestField::Get),
                            2 => Result::Ok(KvsRequestField::Gossip),
                            _ => panic!(),
                        }
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(match v {
                            "Put" => KvsRequestField::Put,
                            "_Get" => KvsRequestField::Get,
                            "Gossip" => KvsRequestField::Gossip,
                            _ => panic!(),
                        })
                    }
                }
                impl<'de> Deserialize<'de> for KvsRequestField {
                    #[inline]
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        Deserializer::deserialize_identifier(deserializer, KVSRequest2FieldVisitor)
                    }
                }

                Ok(match data.variant()? {
                    (KvsRequestField::Put, _variant) => {
                        todo!()
                    }
                    (KvsRequestField::Get, _variant) => {
                        todo!()
                    }
                    (KvsRequestField::Gossip, variant) => variant.struct_variant(
                        &["key", "marker", "buffer"],
                        KvsRequestGossipVisitor {
                            collector: self.collector,
                        },
                    )?,
                })
            }
        }

        deserializer.deserialize_enum(
            "KvsRequest",
            &["Put", "Get", "Gossip"],
            KvsRequestVisitor {
                collector: self.collector,
            },
        )
    }
}

struct KvsRequestGossipVisitor {
    collector: Rc<RefCell<BufferPool>>,
}
impl<'de> Visitor<'de> for KvsRequestGossipVisitor {
    type Value = KvsRequest;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("KvsRequest::Gossip")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let key: u64 = seq.next_element()?.unwrap();
        let marker: u128 = seq.next_element()?.unwrap();
        let buffer: AutoReturnBuffer = seq
            .next_element_seed(AutoReturnBufferDeserializer {
                collector: self.collector,
            })?
            .unwrap();

        Ok(KvsRequest::Gossip {
            key,
            reg: DomPair::new_from(Max::new(marker), Fake::new(buffer)),
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut key = None;
        let mut marker = None;
        let mut buffer = None;

        loop {
            let k: Option<&'de str> = map.next_key()?;
            match k {
                Some("key") => {
                    key = Some(map.next_value()?);
                }
                Some("marker") => {
                    marker = Some(map.next_value()?);
                }
                Some("buffer") => {
                    buffer = Some(map.next_value_seed(AutoReturnBufferDeserializer {
                        collector: self.collector.clone(),
                    })?);
                }
                Some(&_) => panic!(),
                None => break,
            }
        }

        Ok(KvsRequest::Gossip {
            key: key.unwrap(),
            reg: DomPair::new(Max::new(marker.unwrap()), Fake::new(buffer.unwrap())),
        })
    }
}
