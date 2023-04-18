mod serialization;
#[cfg(test)]
mod test;

pub use serialization::KvsRequestDeserializer;

use crate::buffer_pool::AutoReturnBuffer;
use hydroflow::lang::{
    lattice::{last_write_wins::LastWriteWinsRepr, set_union::SetUnionRepr, LatticeRepr},
    tag,
};

pub type MyLastWriteWins = LastWriteWinsRepr<u128, AutoReturnBuffer>;
pub type MyLastWriteWinsRepr = <LastWriteWinsRepr<u128, AutoReturnBuffer> as LatticeRepr>::Repr;
pub type SetUnion = SetUnionRepr<tag::HASH_SET, (Vec<u8>, usize)>;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum KvsRequest {
    Put { key: u64, value: AutoReturnBuffer },
    _Get { key: u64 },
    Gossip { key: u64, reg: MyLastWriteWinsRepr },
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum KvsResponse {
    _PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyLastWriteWinsRepr },
}
