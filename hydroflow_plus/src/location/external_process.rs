use std::marker::PhantomData;

use hydroflow::bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{Location, LocationId, NoTick};
use crate::builder::FlowState;
use crate::ir::{HfPlusNode, HfPlusSource};
use crate::{Stream, Unbounded};

pub struct ExternalBytesPort {
    pub(crate) process_id: usize,
    pub(crate) port_id: usize,
}

pub struct ExternalBincodeSink<T: Serialize> {
    pub(crate) process_id: usize,
    pub(crate) port_id: usize,
    pub(crate) _phantom: PhantomData<T>,
}

pub struct ExternalBincodeStream<T: DeserializeOwned> {
    pub(crate) process_id: usize,
    pub(crate) port_id: usize,
    pub(crate) _phantom: PhantomData<T>,
}

pub struct ExternalProcess<'a, P> {
    pub(crate) id: usize,

    pub(crate) flow_state: FlowState,

    pub(crate) _phantom: PhantomData<&'a &'a mut P>,
}

impl<P> Clone for ExternalProcess<'_, P> {
    fn clone(&self) -> Self {
        ExternalProcess {
            id: self.id,
            flow_state: self.flow_state.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, P> Location<'a> for ExternalProcess<'a, P> {
    fn id(&self) -> LocationId {
        LocationId::ExternalProcess(self.id)
    }

    fn flow_state(&self) -> &FlowState {
        &self.flow_state
    }

    fn is_top_level() -> bool {
        true
    }
}

impl<'a, P> ExternalProcess<'a, P> {
    pub fn source_external_bytes<L: Location<'a> + NoTick>(
        &self,
        to: &L,
    ) -> (ExternalBytesPort, Stream<Bytes, L, Unbounded>) {
        let next_external_port_id = {
            let mut flow_state = self.flow_state.borrow_mut();
            let id = flow_state.next_external_out;
            flow_state.next_external_out += 1;
            id
        };

        (
            ExternalBytesPort {
                process_id: self.id,
                port_id: next_external_port_id,
            },
            Stream::new(
                to.clone(),
                HfPlusNode::Persist(Box::new(HfPlusNode::Network {
                    from_location: LocationId::ExternalProcess(self.id),
                    from_key: Some(next_external_port_id),
                    to_location: to.id(),
                    to_key: None,
                    serialize_pipeline: None,
                    instantiate_fn: crate::ir::DebugInstantiate::Building(),
                    deserialize_pipeline: Some(syn::parse_quote!(map(|b| b.unwrap().freeze()))),
                    input: Box::new(HfPlusNode::Source {
                        source: HfPlusSource::ExternalNetwork(),
                        location_kind: LocationId::ExternalProcess(self.id),
                    }),
                })),
            ),
        )
    }

    pub fn source_external_bincode<L: Location<'a> + NoTick, T: Serialize + DeserializeOwned>(
        &self,
        to: &L,
    ) -> (ExternalBincodeSink<T>, Stream<T, L, Unbounded>) {
        let next_external_port_id = {
            let mut flow_state = self.flow_state.borrow_mut();
            let id = flow_state.next_external_out;
            flow_state.next_external_out += 1;
            id
        };

        (
            ExternalBincodeSink {
                process_id: self.id,
                port_id: next_external_port_id,
                _phantom: PhantomData,
            },
            Stream::new(
                to.clone(),
                HfPlusNode::Persist(Box::new(HfPlusNode::Network {
                    from_location: LocationId::ExternalProcess(self.id),
                    from_key: Some(next_external_port_id),
                    to_location: to.id(),
                    to_key: None,
                    serialize_pipeline: None,
                    instantiate_fn: crate::ir::DebugInstantiate::Building(),
                    deserialize_pipeline: Some(crate::stream::deserialize_bincode::<T>(None)),
                    input: Box::new(HfPlusNode::Source {
                        source: HfPlusSource::ExternalNetwork(),
                        location_kind: LocationId::ExternalProcess(self.id),
                    }),
                })),
            ),
        )
    }
}
