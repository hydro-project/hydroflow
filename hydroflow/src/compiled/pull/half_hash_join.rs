use std::{collections::HashMap, marker::PhantomData};

use crate::lang::lattice::{LatticeRepr, Merge};

pub struct HalfHashJoinState<K, L>
where
    L: LatticeRepr,
{
    tab: HashMap<K, L::Repr>,
}

impl<K, L> Default for HalfHashJoinState<K, L>
where
    L: LatticeRepr,
{
    fn default() -> Self {
        Self {
            tab: HashMap::new(),
        }
    }
}

pub struct HalfHashJoin<'a, K, Buf, L, Update, Stream, StreamV>
where
    K: Eq + std::hash::Hash,
    L: LatticeRepr + Merge<Update>,
    Update: LatticeRepr,
    Buf: Iterator<Item = (K, Update::Repr)>,
    Stream: Iterator<Item = (K, StreamV)>,
{
    buf: Buf,
    stream: Stream,
    state: &'a mut HalfHashJoinState<K, L>,
    _marker: PhantomData<Update>,
}

impl<'a, K, Buf, L, Update, Stream, StreamV> Iterator
    for HalfHashJoin<'a, K, Buf, L, Update, Stream, StreamV>
where
    K: Eq + std::hash::Hash,
    L: LatticeRepr + Merge<Update>,
    L::Repr: Default,
    Update: LatticeRepr,
    Buf: Iterator<Item = (K, Update::Repr)>,
    Stream: Iterator<Item = (K, StreamV)>,
{
    type Item = (K, StreamV, L::Repr);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, v) in &mut self.buf {
            <L as Merge<Update>>::merge(self.state.tab.entry(k).or_default(), v);
        }

        for (k, v) in &mut self.stream {
            if let Some(vals) = self.state.tab.get(&k) {
                return Some((k, v, vals.clone()));
            }
        }
        None
    }
}
impl<'a, K, Buf, L, Update, Stream, StreamV> HalfHashJoin<'a, K, Buf, L, Update, Stream, StreamV>
where
    K: Eq + std::hash::Hash,
    Buf: Iterator<Item = (K, Update::Repr)>,
    Stream: Iterator<Item = (K, StreamV)>,
    Update: LatticeRepr,
    L: LatticeRepr + Merge<Update>,
    L::Repr: Clone,
{
    pub fn new(buf: Buf, stream: Stream, state: &'a mut HalfHashJoinState<K, L>) -> Self {
        Self {
            buf,
            stream,
            state,
            _marker: PhantomData,
        }
    }
}
