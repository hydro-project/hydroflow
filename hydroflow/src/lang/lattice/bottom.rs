use std::cmp::Ordering;

use super::{Compare, Convert, Debottom, Lattice, LatticeRepr, Merge};

pub struct BottomRepr<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> LatticeRepr for BottomRepr<Lr> {
    type Lattice = Lr::Lattice;
    type Repr = Option<Lr::Repr>;
}

impl<SelfLr, DeltaLr, L> Merge<BottomRepr<DeltaLr>> for BottomRepr<SelfLr>
where
    SelfLr: LatticeRepr<Lattice = L> + Merge<DeltaLr>,
    DeltaLr: LatticeRepr<Lattice = L> + Convert<SelfLr>,
    L: Lattice,
{
    fn merge(this: &mut Self::Repr, delta: <BottomRepr<DeltaLr> as LatticeRepr>::Repr) -> bool {
        match (this, delta) {
            (None, None) => false,
            (Some(_), None) => false,
            (this @ None, Some(delta_inner)) => {
                *this = Some(<DeltaLr as Convert<SelfLr>>::convert(delta_inner));
                true
            }
            (Some(this_inner), Some(delta_inner)) => {
                <SelfLr as Merge<DeltaLr>>::merge(this_inner, delta_inner)
            }
        }
    }
}

impl<SelfLr, TargetLr, L> Convert<BottomRepr<TargetLr>> for BottomRepr<SelfLr>
where
    SelfLr: LatticeRepr<Lattice = L> + Convert<TargetLr>,
    TargetLr: LatticeRepr<Lattice = L>,
    L: Lattice,
{
    fn convert(this: Self::Repr) -> <BottomRepr<TargetLr> as LatticeRepr>::Repr {
        this.map(<SelfLr as Convert<TargetLr>>::convert)
    }
}

impl<SelfLr, OtherLr, L> Compare<BottomRepr<OtherLr>> for BottomRepr<SelfLr>
where
    SelfLr: LatticeRepr<Lattice = L> + Compare<OtherLr>,
    OtherLr: LatticeRepr<Lattice = L>,
    L: Lattice,
{
    fn compare(
        this: &Self::Repr,
        other: &<BottomRepr<OtherLr> as LatticeRepr>::Repr,
    ) -> Option<Ordering> {
        match (this, other) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (Some(this_inner), Some(other_inner)) => {
                <SelfLr as Compare<OtherLr>>::compare(this_inner, other_inner)
            }
        }
    }
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
