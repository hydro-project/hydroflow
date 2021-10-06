use crate::lattice::LatticeRepr;
use crate::hide::{Hide, Qualifier};

pub trait BinaryMorphism {
    type InLatReprA: LatticeRepr;
    type InLatReprB: LatticeRepr;
    type OutLatRepr: LatticeRepr;

    fn call<Y: Qualifier>(
        &self,
        item_a: Hide<Y, Self::InLatReprA>, item_b: Hide<Y, Self::InLatReprB>
    )
        -> Hide<Y, Self::OutLatRepr>;
}

mod partitioned;
pub use partitioned::*;

mod cartesian_product;
pub use cartesian_product::*;

mod table_product;
pub use table_product::*;

mod pair;
pub use pair::*;