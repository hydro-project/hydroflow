use std::hash::Hash;

use crate::hide::{Hide, Qualifier};
use crate::lattice::map_union::{MapUnion, MapUnionRepr};
use crate::lattice::LatticeRepr;
use crate::tag;

use super::Morphism;

pub struct HashPartitioned<Lr, K: Eq + Hash + Clone, F: Morphism>
where
    Lr: LatticeRepr<Lattice = MapUnion<K, <F::InLatRepr as LatticeRepr>::Lattice>>,
    Lr::Repr: IntoIterator<Item = (K, <F::InLatRepr as LatticeRepr>::Repr)>,
{
    func: F,
    _phantom: std::marker::PhantomData<(K, Lr)>,
}

impl<Lr, K: Eq + Hash + Clone, F: Morphism> HashPartitioned<Lr, K, F>
where
    Lr: LatticeRepr<Lattice = MapUnion<K, <F::InLatRepr as LatticeRepr>::Lattice>>,
    Lr::Repr: IntoIterator<Item = (K, <F::InLatRepr as LatticeRepr>::Repr)>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr, K: Eq + Hash + Clone, F: Morphism> Morphism for HashPartitioned<Lr, K, F>
where
    Lr: LatticeRepr<Lattice = MapUnion<K, <F::InLatRepr as LatticeRepr>::Lattice>>,
    Lr::Repr: IntoIterator<Item = (K, <F::InLatRepr as LatticeRepr>::Repr)>,
{
    type InLatRepr = Lr;
    type OutLatRepr = MapUnionRepr<tag::HASH_MAP, K, F::OutLatRepr>;

    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        let item = item.into_reveal();

        let out = item
            .into_iter()
            .map(|(k, val)| {
                let hide = self.func.call::<Y>(Hide::new(val));
                (k, hide.into_reveal())
            })
            .collect();
        Hide::new(out)
    }
}
