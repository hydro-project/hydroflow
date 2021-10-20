use crate::hide::{Hide, Qualifier};
use crate::lattice::pair::PairRepr;
use crate::lattice::LatticeRepr;

use super::BinaryMorphism;

pub struct Pair<A: LatticeRepr, B: LatticeRepr>(std::marker::PhantomData<(A, B)>);

impl<A: LatticeRepr, B: LatticeRepr> Pair<A, B> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<A: LatticeRepr, B: LatticeRepr> Default for Pair<A, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: LatticeRepr, B: LatticeRepr> BinaryMorphism for Pair<A, B> {
    type InLatReprA = A;
    type InLatReprB = B;
    type OutLatRepr = PairRepr<A, B>;

    fn call<Y: Qualifier>(
        &self,
        item_a: Hide<Y, Self::InLatReprA>,
        item_b: Hide<Y, Self::InLatReprB>,
    ) -> Hide<Y, Self::OutLatRepr> {
        Hide::new((item_a.into_reveal(), item_b.into_reveal()))
    }
}
