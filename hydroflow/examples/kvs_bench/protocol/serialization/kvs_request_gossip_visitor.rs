use crate::{
    buffer_pool::{AutoReturnBuffer, BufferPool, OptionalAutoReturnBufferDeserializer},
    protocol::{KvsRequest, MyLastWriteWins},
};
use lattices::{bottom::Bottom, fake::Fake, ord::Max};
use serde::de::{SeqAccess, Visitor};
use std::{cell::RefCell, rc::Rc};

pub struct KvsRequestGossipVisitor {
    pub collector: Rc<RefCell<BufferPool>>,
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
        let buffer = seq
            .next_element_seed(OptionalAutoReturnBufferDeserializer {
                collector: self.collector,
            })?
            .unwrap();

        let val = if let Some(buffer) = buffer {
            Bottom::new(Fake::new(buffer))
        } else {
            Bottom::default()
        };

        Ok(KvsRequest::Gossip {
            key,
            reg: MyLastWriteWins::new(Max::new(marker), val),
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut key = None;
        let mut marker = None;
        let mut buffer: Option<Option<AutoReturnBuffer>> = None;

        loop {
            let k: Option<String> = map.next_key()?;
            if let Some(k) = k {
                match k.as_str() {
                    "key" => {
                        key = Some(map.next_value()?);
                    }
                    "marker" => {
                        marker = Some(map.next_value()?);
                    }
                    "buffer" => {
                        buffer =
                            Some(map.next_value_seed(OptionalAutoReturnBufferDeserializer {
                                collector: self.collector.clone(),
                            })?);
                    }
                    _ => panic!(),
                }
            } else {
                break;
            }
        }

        assert!(key.is_some());
        assert!(marker.is_some());
        assert!(buffer.is_some());

        let val = if let Some(buffer) = buffer.unwrap() {
            Bottom::new(Fake::new(buffer))
        } else {
            Bottom::default()
        };

        Ok(KvsRequest::Gossip {
            key: key.unwrap(),
            reg: MyLastWriteWins::new(Max::new(marker.unwrap()), val),
        })
    }
}
