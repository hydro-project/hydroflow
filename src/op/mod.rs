use std::borrow::Cow;

use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::LatticeRepr;

pub trait Op {
    // TODO: separate REPRs for Delta and Cumul.
    type ILatRepr: LatticeRepr;
    type OLatRepr: LatticeRepr;

    type State: LatticeRepr;
}

pub trait OpDelta: Op {
    #[must_use]
    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::ILatRepr>>)
        -> Cow<'h, Hide<Delta, Self::OLatRepr>>;
}

pub trait OpCumul: Op {
    #[must_use]
    fn get_cumul<'h>(state: &'h mut Hide<Cumul, Self::State>)
        -> Cow<'h, Hide<Cumul, Self::OLatRepr>>;
}

pub mod identity;
pub mod state_merge;
pub mod static_iter;
