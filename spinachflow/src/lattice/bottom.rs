use super::{LatticeRepr, Debottom};

pub struct BottomRepr<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> LatticeRepr for BottomRepr<Lr> {
    type Lattice = Lr::Lattice;
    type Repr = Option<Lr::Repr>;
}

impl<Lr: LatticeRepr> Debottom for BottomRepr<Lr> {
    fn is_bottom(this: &Self::Repr) -> bool {
        this.is_none()
    }

    type DebottomLr = Lr;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        this
    }
}
