use std::borrow::Cow;

use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::{Lattice, LatticeRepr};

pub trait Op {
    // TODO: separate REPRs for Delta and Cumul.
    type Lat: Lattice;

    type State: LatticeRepr;
}

pub trait OpDelta: Op {
    type LatReprDeltaIn:  LatticeRepr;
    type LatReprDeltaOut: LatticeRepr<Lattice = Self::Lat>;

    #[must_use]
    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
        -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>>;
}

pub trait OpCumul: Op {
    type LatReprCumulOut: LatticeRepr<Lattice = Self::Lat>;

    #[must_use]
    fn get_cumul<'h>(state: &'h mut Hide<Cumul, Self::State>)
        -> Cow<'h, Hide<Cumul, Self::LatReprCumulOut>>;
}

pub mod identity;
pub mod state_merge;
pub mod split_binary;
