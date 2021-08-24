use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::LatticeRepr;

pub trait Op {
    type ILatRepr: LatticeRepr;
    type OLatRepr: LatticeRepr;

    type State: LatticeRepr;
}

pub trait OpDelta: Op {
    fn get_delta<'h>(state: Hide<Cumul, Self::State>, input: Hide<Delta, Self::ILatRepr>)
        -> Hide<Delta, Self::OLatRepr>;
}

pub trait OpCumul: Op {
    fn get_value<'h>(state: Hide<Cumul, Self::State>)
        -> Hide<Cumul, Self::OLatRepr>;
}

pub mod state_merge;
