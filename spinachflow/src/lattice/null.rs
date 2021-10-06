use super::{Lattice, LatticeRepr, Merge, Convert, Debottom};

pub enum Null {}
impl Lattice for Null {}

pub enum NullRepr {}

impl LatticeRepr for NullRepr {
    type Lattice = Null;
    type Repr = ();
}

impl Merge<NullRepr> for NullRepr {
    fn merge(_this: &mut <NullRepr as LatticeRepr>::Repr, _delta: <NullRepr as LatticeRepr>::Repr) -> bool {
        false
    }
}

impl Convert<NullRepr> for NullRepr {
    fn convert(this: Self::Repr) -> <NullRepr as LatticeRepr>::Repr {
        this
    }
}

impl Debottom for NullRepr {
    fn is_bottom(_this: &Self::Repr) -> bool {
        true
    }

    type DebottomLr = NullRepr;
    fn debottom(_this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        None
    }
}
