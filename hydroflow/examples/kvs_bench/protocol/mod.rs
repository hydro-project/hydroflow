mod serialization;
#[cfg(test)]
mod test;

pub use serialization::KvsRequestDeserializer;

use crate::buffer_pool::AutoReturnBuffer;
use hydroflow::lang::lattice2::set_union::SetUnion;
use hydroflow::lang::lattice2::{dom_pair::DomPair, fake::Fake, ord::Max};
use hydroflow::lang::tag;

pub type MyLastWriteWinsRepr = DomPair<Max<u128>, Fake<AutoReturnBuffer>>;
pub type MySetUnion = SetUnion<tag::HASH_SET, (Vec<u8>, usize)>;

#[derive(Clone, Debug)]
pub enum KvsRequest {
    Put { key: u64, value: AutoReturnBuffer },
    _Get { key: u64 },
    Gossip { key: u64, reg: MyLastWriteWinsRepr },
}

#[derive(Clone, Debug)]
pub enum KvsResponse {
    _PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyLastWriteWinsRepr },
}
