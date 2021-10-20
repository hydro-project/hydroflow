use std::iter::FromIterator;

use crate::hide::{Hide, Qualifier};
use crate::lattice::map_union::MapUnion;
use crate::lattice::set_union::SetUnion;
use crate::lattice::LatticeRepr;

use super::BinaryMorphism;

pub struct TableProduct<SetLr, K: Clone, ValLr, TargetLr>
where
    SetLr: LatticeRepr<Lattice = SetUnion<K>>,
    SetLr::Repr: IntoIterator<Item = K>,

    ValLr: LatticeRepr,

    TargetLr: LatticeRepr<Lattice = MapUnion<K, ValLr::Lattice>>,
    TargetLr::Repr: FromIterator<(K, ValLr::Repr)>,
{
    _phantom: std::marker::PhantomData<(SetLr, K, ValLr, TargetLr)>,
}

impl<SetLr, K: Clone, ValLr, TargetLr> TableProduct<SetLr, K, ValLr, TargetLr>
where
    SetLr: LatticeRepr<Lattice = SetUnion<K>>,
    SetLr::Repr: IntoIterator<Item = K>,

    ValLr: LatticeRepr,

    TargetLr: LatticeRepr<Lattice = MapUnion<K, ValLr::Lattice>>,
    TargetLr::Repr: FromIterator<(K, ValLr::Repr)>,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<SetLr, K: Clone, ValLr, TargetLr> Default for TableProduct<SetLr, K, ValLr, TargetLr>
where
    SetLr: LatticeRepr<Lattice = SetUnion<K>>,
    SetLr::Repr: IntoIterator<Item = K>,

    ValLr: LatticeRepr,

    TargetLr: LatticeRepr<Lattice = MapUnion<K, ValLr::Lattice>>,
    TargetLr::Repr: FromIterator<(K, ValLr::Repr)>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<SetLr, K: Clone, ValLr, TargetLr> BinaryMorphism for TableProduct<SetLr, K, ValLr, TargetLr>
where
    SetLr: LatticeRepr<Lattice = SetUnion<K>>,
    SetLr::Repr: IntoIterator<Item = K>,

    ValLr: LatticeRepr,

    TargetLr: LatticeRepr<Lattice = MapUnion<K, ValLr::Lattice>>,
    TargetLr::Repr: FromIterator<(K, ValLr::Repr)>,
{
    type InLatReprA = SetLr;
    type InLatReprB = ValLr;
    type OutLatRepr = TargetLr;

    fn call<Y: Qualifier>(
        &self,
        item_a: Hide<Y, Self::InLatReprA>,
        item_b: Hide<Y, Self::InLatReprB>,
    ) -> Hide<Y, Self::OutLatRepr> {
        let out = item_a
            .into_reveal()
            .into_iter()
            .map(|k| (k, item_b.reveal_ref().clone()))
            .collect();
        Hide::new(out)
    }
}
