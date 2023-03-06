use hydroflow::lang::lattice::ord::MaxRepr;
use hydroflow::lang::lattice::LatticeRepr;
use hydroflow::lang::{lattice::map_union::MapUnionRepr, tag};

use serde::{Deserialize, Serialize};

pub type VecClock = MapUnionRepr<tag::HASH_MAP, String, MaxRepr<usize>>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct EchoMsg {
    pub payload: String,
    pub vc: <VecClock as LatticeRepr>::Repr,
}
