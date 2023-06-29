use crate::{IsBot, IsTop, LatticeFrom, LatticeOrd, Merge};

impl Merge<()> for () {
    fn merge(&mut self, _other: ()) -> bool {
        false
    }
}

impl LatticeOrd for () {}

impl LatticeFrom<()> for () {
    fn lattice_from(other: ()) -> Self {
        other
    }
}

impl IsBot for () {
    fn is_bot(&self) -> bool {
        true
    }
}

impl IsTop for () {
    fn is_top(&self) -> bool {
        true
    }
}
