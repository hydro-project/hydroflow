use crate::hide::{Delta, Hide, Qualifier};
use crate::lattice::LatticeRepr;

use super::Morphism;

pub struct ClosureMorphism<In: LatticeRepr, Out: LatticeRepr, F>
where
    F: Fn(Hide<Delta, In>) -> Hide<Delta, Out>,
{
    func: F,
    _phantom: std::marker::PhantomData<(In, Out)>,
}

impl<In: LatticeRepr, Out: LatticeRepr, F> ClosureMorphism<In, Out, F>
where
    F: Fn(Hide<Delta, In>) -> Hide<Delta, Out>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<In: LatticeRepr, Out: LatticeRepr, F> Morphism for ClosureMorphism<In, Out, F>
where
    F: Fn(Hide<Delta, In>) -> Hide<Delta, Out>,
{
    type InLatRepr = In;
    type OutLatRepr = Out;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        (self.func)(item.into_delta()).into_qualifier_reveal()
    }
}
