use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use stageleft::Quoted;

use super::{Location, LocationId};
use crate::builder::{ClusterIds, ClusterSelfId, FlowState};

pub struct Cluster<'a, C> {
    pub(crate) id: usize,
    pub(crate) flow_state: FlowState,
    pub(crate) _phantom: PhantomData<&'a &'a mut C>,
}

impl<'a, C> Cluster<'a, C> {
    pub fn self_id(&self) -> impl Quoted<'a, ClusterId<C>> + Copy + 'a {
        ClusterSelfId {
            id: self.id,
            _phantom: PhantomData,
        }
    }

    pub fn members(&self) -> impl Quoted<'a, &'a Vec<ClusterId<C>>> + Copy + 'a {
        ClusterIds {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<C> Clone for Cluster<'_, C> {
    fn clone(&self) -> Self {
        Cluster {
            id: self.id,
            flow_state: self.flow_state.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, C> Location<'a> for Cluster<'a, C> {
    fn id(&self) -> LocationId {
        LocationId::Cluster(self.id)
    }

    fn flow_state(&self) -> &FlowState {
        &self.flow_state
    }

    fn make_from(id: LocationId, flow_state: FlowState) -> Self {
        match id {
            LocationId::Cluster(id) => Cluster {
                id,
                flow_state,
                _phantom: PhantomData,
            },
            _ => panic!(),
        }
    }
}

#[repr(transparent)]
pub struct ClusterId<C> {
    pub raw_id: u32,
    pub(crate) _phantom: PhantomData<C>,
}

impl<C> Debug for ClusterId<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClusterId::<{}>({})",
            std::any::type_name::<C>(),
            self.raw_id
        )
    }
}

impl<C> Display for ClusterId<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClusterId::<{}>({})",
            std::any::type_name::<C>(),
            self.raw_id
        )
    }
}

impl<C> Clone for ClusterId<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ClusterId<C> {}

impl<C> Serialize for ClusterId<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.raw_id.serialize(serializer)
    }
}

impl<'de, C> Deserialize<'de> for ClusterId<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(|id| ClusterId {
            raw_id: id,
            _phantom: PhantomData,
        })
    }
}

impl<C> PartialEq for ClusterId<C> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_id == other.raw_id
    }
}

impl<C> Eq for ClusterId<C> {}

impl<C> PartialOrd for ClusterId<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<C> Ord for ClusterId<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw_id.cmp(&other.raw_id)
    }
}

impl<C> Hash for ClusterId<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw_id.hash(state)
    }
}

impl<C> ClusterId<C> {
    pub fn from_raw(id: u32) -> Self {
        ClusterId {
            raw_id: id,
            _phantom: PhantomData,
        }
    }
}
