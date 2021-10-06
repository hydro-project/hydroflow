use std::borrow::Cow;

use crate::hide::{Hide};
use crate::lattice::{LatticeRepr};
use crate::props::{OpProps};

pub trait OpInternal<Props: OpProps> {
    type LatReprIn:  LatticeRepr;
    type LatReprOut: LatticeRepr;

    type PropsOut: OpProps;

    #[must_use]
    fn run<'h>(&'h mut self, element: Cow<'h, Hide<Self::LatReprIn, Props>>)
        -> Cow<'h, Hide<Self::LatReprOut, Self::PropsOut>>;
}


pub trait Op<Props: OpProps> {
    type LatReprIn:  LatticeRepr;
    type LatReprOut: LatticeRepr;

    type PropsOut: OpProps;

    #[must_use]
    fn run<'h>(&'h mut self, element: Cow<'h, Hide<Self::LatReprIn, Props>>)
        -> Cow<'h, Hide<Self::LatReprOut, Self::PropsOut>>;
}

// pub trait OpImpl<const META: OpProps> {
//     fn get() {}
// }

// pub trait Op<const META: OpProps> {
//     fn get() {}
// }


// pub trait OpDelta: Op {
//     type LatReprDeltaIn:  LatticeRepr;
//     type LatReprDeltaOut: LatticeRepr<Lattice = Self::Lat>;

//     #[must_use]
//     fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
//         -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>>;
// }

// pub trait OpCumul: Op {
//     type LatReprCumulOut: LatticeRepr<Lattice = Self::Lat>;

//     #[must_use]
//     fn get_cumul<'h>(state: &'h mut Hide<Cumul, Self::State>)
//         -> Cow<'h, Hide<Cumul, Self::LatReprCumulOut>>;
// }
