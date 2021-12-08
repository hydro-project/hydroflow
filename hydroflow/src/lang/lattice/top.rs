use super::{LatticeRepr, Top};

pub struct TopRepr<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> LatticeRepr for TopRepr<Lr> {
    type Lattice = Lr::Lattice;
    type Repr = Option<Lr::Repr>;
}

impl<Lr: LatticeRepr> Top for TopRepr<Lr> {
    fn is_top(this: &Self::Repr) -> bool {
        this.is_none()
    }

    fn top() -> Self::Repr {
        None
    }
}
