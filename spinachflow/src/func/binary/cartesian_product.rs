use std::iter::FromIterator;

use crate::hide::{Hide, Qualifier};
use crate::lattice::set_union::SetUnion;
use crate::lattice::LatticeRepr;

use super::BinaryMorphism;

pub struct CartesianProduct<A, AItem, B, BItem, O>
where
    A: LatticeRepr<Lattice = SetUnion<AItem>>,
    B: LatticeRepr<Lattice = SetUnion<BItem>>,
    A::Repr: IntoIterator<Item = AItem>,
    B::Repr: IntoIterator<Item = BItem>,
    AItem: Clone,
    O: LatticeRepr<Lattice = SetUnion<(AItem, BItem)>>,
    O::Repr: FromIterator<(AItem, BItem)>,
{
    _phantom: std::marker::PhantomData<(A, AItem, B, BItem, O)>,
}

impl<A, AItem, B, BItem, O> CartesianProduct<A, AItem, B, BItem, O>
where
    A: LatticeRepr<Lattice = SetUnion<AItem>>,
    B: LatticeRepr<Lattice = SetUnion<BItem>>,
    A::Repr: IntoIterator<Item = AItem>,
    B::Repr: IntoIterator<Item = BItem>,
    AItem: Clone,
    O: LatticeRepr<Lattice = SetUnion<(AItem, BItem)>>,
    O::Repr: FromIterator<(AItem, BItem)>,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A, AItem, B, BItem, O> Default for CartesianProduct<A, AItem, B, BItem, O>
where
    A: LatticeRepr<Lattice = SetUnion<AItem>>,
    B: LatticeRepr<Lattice = SetUnion<BItem>>,
    A::Repr: IntoIterator<Item = AItem>,
    B::Repr: IntoIterator<Item = BItem>,
    AItem: Clone,
    O: LatticeRepr<Lattice = SetUnion<(AItem, BItem)>>,
    O::Repr: FromIterator<(AItem, BItem)>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A, AItem, B, BItem, O> BinaryMorphism for CartesianProduct<A, AItem, B, BItem, O>
where
    A: LatticeRepr<Lattice = SetUnion<AItem>>,
    B: LatticeRepr<Lattice = SetUnion<BItem>>,
    A::Repr: IntoIterator<Item = AItem>,
    B::Repr: IntoIterator<Item = BItem>,
    AItem: Clone,
    O: LatticeRepr<Lattice = SetUnion<(AItem, BItem)>>,
    O::Repr: FromIterator<(AItem, BItem)>,
{
    type InLatReprA = A;
    type InLatReprB = B;
    type OutLatRepr = O;

    fn call<Y: Qualifier>(
        &self,
        item_a: Hide<Y, Self::InLatReprA>,
        item_b: Hide<Y, Self::InLatReprB>,
    ) -> Hide<Y, Self::OutLatRepr> {
        let out = item_a
            .into_reveal()
            .into_iter()
            .flat_map(|a| {
                item_b
                    .clone()
                    .into_reveal()
                    .into_iter()
                    .map(move |b| (a.clone(), b))
            })
            .collect();
        Hide::new(out)
    }
}
