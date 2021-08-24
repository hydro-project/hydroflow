use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::LatticeRepr;

pub trait Op {
    type ILatRepr: LatticeRepr;
    type OLatRepr: LatticeRepr;

    type State;
}

pub trait OpDelta: Op {
    fn get_delta<'h>(state: &'h mut Self::State, input: Hide<'h, Delta, Self::ILatRepr>)
        -> Hide<'h, Delta, Self::OLatRepr>;
}

pub trait OpCumul: Op {
    fn get_value<'a>(state: &'a mut Self::State)
        -> Hide<'a, Cumul, Self::OLatRepr>;
}

pub mod state_merge;
