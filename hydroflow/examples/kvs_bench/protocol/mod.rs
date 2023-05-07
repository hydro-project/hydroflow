mod serialization;

#[cfg(test)]
mod test;

pub use serialization::KvsRequestDeserializer;

use crate::buffer_pool::AutoReturnBuffer;
use lattices::bottom::Bottom;
use lattices::set_union::SetUnionHashSet;
use lattices::{dom_pair::DomPair, fake::Fake, ord::Max};

pub type NodeId = usize;

pub type MyLastWriteWins = DomPair<Max<u128>, Bottom<Fake<AutoReturnBuffer>>>;
pub type MySetUnion = SetUnionHashSet<(NodeId, usize)>;

#[derive(Clone, Debug)]
pub enum KvsRequest {
    Put { key: u64, value: AutoReturnBuffer },
    Get { key: u64 },
    Gossip { key: u64, reg: MyLastWriteWins },
    Delete { key: u64 },
}

#[derive(Clone, Debug)]
pub enum KvsResponse {
    _PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyLastWriteWins },
}
