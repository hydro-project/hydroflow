use super::Pusherator;

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::lang::lattice::{Convert, LatticeRepr, Merge};

// TODO(mingwei): Use map-union lattice to represent groups?
pub struct GroupBy<K, V, V2, O>
where
    K: Eq + std::hash::Hash + Clone,
    V: LatticeRepr<Lattice = V2::Lattice> + Merge<V2>,
    V2: LatticeRepr + Convert<V>,
    O: Pusherator<Item = (K, V::Repr)>,
{
    contents: HashMap<K, V::Repr>,
    out: O,
    _phantom: std::marker::PhantomData<fn(V2)>,
}
impl<K, V, V2, O> Pusherator for GroupBy<K, V, V2, O>
where
    K: Eq + std::hash::Hash + Clone,
    V: LatticeRepr<Lattice = V2::Lattice> + Merge<V2>,
    V2: LatticeRepr + Convert<V>,
    O: Pusherator<Item = (K, V::Repr)>,
{
    type Item = (K, V2::Repr);
    fn give(&mut self, item: Self::Item) {
        // TODO(justin): we need a more coherent understanding of time in order
        // to not emit a ton of extra stuff here.
        if let Some(v) = self.contents.get_mut(&item.0) {
            if V::merge(v, item.1) {
                self.out.give((item.0, v.clone()));
            }
        } else {
            let v = V2::convert(item.1);
            self.contents.insert(item.0.clone(), v.clone());
            self.out.give((item.0, v));
        }
    }
}
impl<K, V, V2, O> GroupBy<K, V, V2, O>
where
    K: Eq + std::hash::Hash + Clone,
    V: LatticeRepr<Lattice = V2::Lattice> + Merge<V2>,
    V2: LatticeRepr + Convert<V>,
    O: Pusherator<Item = (K, V::Repr)>,
{
    pub fn new(out: O) -> Self {
        Self {
            contents: HashMap::new(),
            out,
            _phantom: PhantomData,
        }
    }
}
