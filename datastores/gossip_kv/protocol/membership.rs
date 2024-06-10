use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

/// Information about a member in the cluster.
///
/// A member is a transducer that is part of the cluster. Leaving or failing is a terminal
/// state for a member. When a transducer restarts and rejoins the cluster, it is considered a
/// new member.
///
/// # Generic Parameters
/// -- `A`: The transport of the endpoint on which the protocol is running. In production, this will
/// likely be a `SocketAddr`.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct MemberData<A>
where
    A: Debug + Clone + Eq + Hash + Serialize,
{
    /// The name of the member. Usually, this is a randomly generated identifier, based on the
    /// hostname on which the member is running.
    pub name: String,

    /// The protocols that the member supports.
    pub protocols: Vec<Protocol<A>>,
}

/// A builder for `MemberData`.
pub struct MemberDataBuilder<A>
where
    A: Debug + Clone + Eq + Hash + Serialize,
{
    name: String,
    protocols: Vec<Protocol<A>>,
}

impl<A> MemberDataBuilder<A>
where
    A: Debug + Clone + Eq + Hash + Serialize,
{
    /// Creates a new `MemberDataBuilder`.
    pub fn new(name: String) -> Self {
        MemberDataBuilder {
            name,
            protocols: Vec::new(),
        }
    }

    /// Adds a protocol to the member.
    pub fn add_protocol(mut self, protocol: Protocol<A>) -> Self {
        self.protocols.push(protocol);
        self
    }

    /// Builds the `MemberData`.
    pub fn build(self) -> MemberData<A> {
        MemberData {
            name: self.name,
            protocols: self.protocols,
        }
    }
}

/// A protocol supported by a member.
///
/// # Generic Parameters
/// -- `A`: The transport of the endpoint on which the protocol is running. In production, this will
/// likely be a `SocketAddr`.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Protocol<A> {
    /// The name of the protocol.
    pub name: String,

    /// The endpoint on which the protocol is running.
    pub endpoint: A,
}

impl<A> Protocol<A> {
    /// Creates a new `Protocol`.
    pub fn new(name: String, endpoint: A) -> Self {
        Protocol { name, endpoint }
    }
}
