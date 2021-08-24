use crate::hide::{Hide, Qualifier};

pub mod set_union;
pub mod map_union;
pub mod ord;
pub mod pair;
pub mod dom_pair;
pub mod bottom;
pub mod top;

pub trait Lattice {}

pub trait LatticeRepr {
    type Lattice: Lattice;
    type Repr: Clone;

    fn run(_inst: &mut Self::Repr) {
    }
}

pub trait Merge<Delta: LatticeRepr>: LatticeRepr<Lattice = Delta::Lattice> {
    /// Merge DELTA into THIS. Return TRUE if THIS changed, FALSE if THIS was unchanged.
    fn merge(this: &mut Self::Repr, delta: Delta::Repr) -> bool;

    fn merge_hide<Y: Qualifier, Z: Qualifier>(this: &mut Hide<Y, Self>, delta: Hide<Z, Delta>) -> bool {
        Self::merge(this.reveal_mut(), delta.into_reveal())
    }
}

pub trait Convert<Target: LatticeRepr<Lattice = Self::Lattice>>: LatticeRepr {
    fn convert(this: Self::Repr) -> Target::Repr;

    fn convert_hide<Y: Qualifier>(this: Hide<Y, Self>) -> Hide<Y, Target> {
        Hide::new(Self::convert(this.into_reveal()))
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
