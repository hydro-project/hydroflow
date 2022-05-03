use std::marker::PhantomData;

use crate::lang::lattice::{LatticeRepr, Merge};

#[derive(Debug)]
pub struct BatchJoinState<L>
where
    L: LatticeRepr,
    L::Repr: Default,
{
    state: L::Repr,
}

impl<L> Default for BatchJoinState<L>
where
    L: LatticeRepr,
    L::Repr: Default,
{
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

pub struct BatchJoin<'a, Buf, Stream, L, Update, Tick>
where
    Buf: Iterator<Item = Update::Repr>,
    Stream: Iterator<Item = Tick>,
    Update: LatticeRepr,
    L: LatticeRepr + Merge<Update>,
    L::Repr: Default,
{
    buf: Buf,
    stream: Stream,
    state: &'a mut BatchJoinState<L>,
    _marker: PhantomData<(Update, Tick)>,
}

impl<'a, Buf, Stream, L, Update, Tick> Iterator for BatchJoin<'a, Buf, Stream, L, Update, Tick>
where
    Buf: Iterator<Item = Update::Repr>,
    Stream: Iterator<Item = Tick>,
    Update: LatticeRepr,
    L: LatticeRepr + Merge<Update>,
    L::Repr: Default,
{
    type Item = (Tick, L::Repr);

    fn next(&mut self) -> Option<Self::Item> {
        for p in &mut self.buf {
            <L as Merge<Update>>::merge(&mut self.state.state, p);
        }

        self.stream
            .next()
            .map(|t| (t, std::mem::take(&mut self.state.state)))
    }
}
impl<'a, Buf, Stream, L, Update, Tick> BatchJoin<'a, Buf, Stream, L, Update, Tick>
where
    Buf: Iterator<Item = Update::Repr>,
    Stream: Iterator<Item = Tick>,
    Update: LatticeRepr,
    L: LatticeRepr + Merge<Update>,
    L::Repr: Default,
{
    pub fn new(buf: Buf, stream: Stream, state: &'a mut BatchJoinState<L>) -> Self {
        Self {
            buf,
            stream,
            state,
            _marker: PhantomData,
        }
    }
}
