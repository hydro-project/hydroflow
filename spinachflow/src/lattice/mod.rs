use std::borrow::Cow;
use crate::hide::{Hide};
use crate::props::OpProps;

pub mod set_union;
pub mod map_union;
pub mod sequence;
pub mod ord;
pub mod pair;
pub mod dom_pair;
pub mod bottom;
pub mod top;
pub mod null;

pub trait Lattice {}

pub trait LatticeRepr: 'static {
    type Lattice: Lattice;
    type Repr: Clone;

    fn run(_inst: &mut Self::Repr) {
    }
}

pub trait Merge<Delta: LatticeRepr>: LatticeRepr<Lattice = Delta::Lattice> {
    /// Merge DELTA into THIS. Return TRUE if THIS changed, FALSE if THIS was unchanged.
    fn merge(this: &mut Self::Repr, delta: Delta::Repr) -> bool;

    fn merge_hide<PropsThis: OpProps, PropsDelta: OpProps>(this: &mut Hide<Self, PropsThis>, delta: Hide<Delta, PropsDelta>) -> bool {
        Self::merge(this.reveal_mut(), <Hide<Delta, PropsDelta>>::into_reveal(delta))
    }
}

pub trait Convert<Target: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn convert(this: Self::Repr) -> Target::Repr;

    fn convert_hide<Props: OpProps>(this: Hide<Self, Props>) -> Hide<Target, Props> {
        <Hide<Target, Props>>::new(Self::convert(<Hide<Self, Props>>::into_reveal(this)))
    }

    fn convert_hide_cow<'h, Props: OpProps>(this: Cow<'h, Hide<Self, Props>>) -> Hide<Target, Props>
    where
        Self: Sized,
    {
        // TODO MAKES EXTRA CLONE (into_owned())...
        <Hide<Target, Props>>::new(Self::convert(<Hide<Self, Props>>::into_reveal(this.into_owned())))
    }
}

pub trait Compare<Other: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn compare(this: &Self::Repr, other: &Other::Repr) -> Option<std::cmp::Ordering>;
}

pub trait Debottom: LatticeRepr {
    fn is_bottom(this: &Self::Repr) -> bool;

    type DebottomLr: LatticeRepr<Lattice = Self::Lattice>;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr>;
}

pub trait Top: LatticeRepr {
    fn is_top(this: &Self::Repr) -> bool;
    fn top() -> Self::Repr;
}
