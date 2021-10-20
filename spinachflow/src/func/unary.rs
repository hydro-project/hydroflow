use crate::hide::{Hide, Qualifier, Value};
use crate::lattice::LatticeRepr;

pub trait Monotone {
    type InLatRepr: LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call(&self, item: Hide<Value, Self::InLatRepr>) -> Hide<Value, Self::OutLatRepr>;
}

pub trait Morphism {
    type InLatRepr: LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr>;
}

impl<M: Morphism> Monotone for M {
    type InLatRepr = <Self as Morphism>::InLatRepr;
    type OutLatRepr = <Self as Morphism>::OutLatRepr;
    fn call(&self, item: Hide<Value, Self::InLatRepr>) -> Hide<Value, Self::OutLatRepr> {
        <Self as Morphism>::call(self, item)
    }
}

mod closure;
pub use closure::*;

mod partitioned;
pub use partitioned::*;
