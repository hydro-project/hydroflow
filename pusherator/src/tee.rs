use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct Tee<T, O1, O2>
where
    T: Clone,
    O1: Pusherator<Item = T>,
    O2: Pusherator<Item = T>,
{
    out1: O1,
    out2: O2,
    _marker: PhantomData<T>,
}
impl<T, O1, O2> Pusherator for Tee<T, O1, O2>
where
    T: Clone,
    O1: Pusherator<Item = T>,
    O2: Pusherator<Item = T>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        self.out1.give(item.clone());
        self.out2.give(item);
    }
}
impl<T, O1, O2> Tee<T, O1, O2>
where
    T: Clone,
    O1: Pusherator<Item = T>,
    O2: Pusherator<Item = T>,
{
    pub fn new(out1: O1, out2: O2) -> Self {
        Self {
            out1,
            out2,
            _marker: PhantomData,
        }
    }
}

pub struct TeeBuild<T, O1, P>
where
    T: Clone,
    P: PusheratorBuild<Item = T>,
    O1: Pusherator<Item = T>,
{
    prev: P,
    first_out: O1,
    _marker: PhantomData<T>,
}
impl<T, O1, P> TeeBuild<T, O1, P>
where
    T: Clone,
    P: PusheratorBuild<Item = T>,
    O1: Pusherator<Item = T>,
{
    pub fn new(prev: P, first_out: O1) -> Self {
        Self {
            prev,
            first_out,
            _marker: PhantomData,
        }
    }
}
impl<T, O1, P> PusheratorBuild for TeeBuild<T, O1, P>
where
    T: Clone,
    P: PusheratorBuild<Item = T>,
    O1: Pusherator<Item = T>,
{
    type Item = T;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<Tee<T, O1, O>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(Tee::new(self.first_out, input))
    }
}
