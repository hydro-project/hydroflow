use std::hash::Hash;

use crate::hide::{Hide, Qualifier};
use crate::lattice::map_union::MapUnionRepr;
use crate::tag;

use super::BinaryMorphism;

pub struct HashPartitioned<K: Eq + Hash + Clone, F: BinaryMorphism> {
    func: F,
    _phantom: std::marker::PhantomData<K>,
}

impl<K: Eq + Hash + Clone, F: BinaryMorphism> HashPartitioned<K, F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K: Eq + Hash + Clone, F: BinaryMorphism> BinaryMorphism for HashPartitioned<K, F> {
    type InLatReprA = MapUnionRepr<tag::HASH_MAP, K, F::InLatReprA>;
    type InLatReprB = MapUnionRepr<tag::HASH_MAP, K, F::InLatReprB>;
    type OutLatRepr = MapUnionRepr<tag::HASH_MAP, K, F::OutLatRepr>;

    fn call<Y: Qualifier>(
        &self,
        item_a: Hide<Y, Self::InLatReprA>,
        item_b: Hide<Y, Self::InLatReprB>,
    ) -> Hide<Y, Self::OutLatRepr> {
        let item_a = item_a.into_reveal();
        let item_b = item_b.into_reveal();

        let out = item_a
            .into_iter()
            .filter_map(|(k, val_a)| {
                item_b
                    .get(&k)
                    .map(|val_b| {
                        self.func
                            .call::<Y>(Hide::new(val_a), Hide::new(val_b.clone()))
                    })
                    .map(|hide_out| (k, hide_out.into_reveal()))
            })
            .collect();
        Hide::new(out)
    }
}
