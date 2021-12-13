use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct Partition<T, F, O1, O2>
where
    F: Fn(&T) -> bool,
    O1: Pusherator<Item = T>,
    O2: Pusherator<Item = T>,
{
    out1: O1,
    out2: O2,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, F, O1, O2> Pusherator for Partition<T, F, O1, O2>
where
    F: Fn(&T) -> bool,
    O1: Pusherator<Item = T>,
    O2: Pusherator<Item = T>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        if (self.f)(&item) {
            self.out1.give(item);
        } else {
            self.out2.give(item);
        }
    }
}
impl<T, F, O1, O2> Partition<T, F, O1, O2>
where
    F: Fn(&T) -> bool,
    O1: Pusherator<Item = T>,
    O2: Pusherator<Item = T>,
{
    pub fn new(f: F, out1: O1, out2: O2) -> Self {
        Self {
            out1,
            out2,
            f,
            _marker: PhantomData,
        }
    }
}

pub struct PartitionBuild<T, F, O1, P>
where
    F: Fn(&T) -> bool,
    O1: Pusherator<Item = T>,
    P: PusheratorBuild<Item = T>,
{
    prev: P,
    out1: O1,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, F, O1, P> PartitionBuild<T, F, O1, P>
where
    F: Fn(&T) -> bool,
    O1: Pusherator<Item = T>,
    P: PusheratorBuild<Item = T>,
{
    pub fn new(prev: P, out1: O1, f: F) -> Self {
        Self {
            prev,
            out1,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, F, O1, P> PusheratorBuild for PartitionBuild<T, F, O1, P>
where
    F: Fn(&T) -> bool,
    O1: Pusherator<Item = T>,
    P: PusheratorBuild<Item = T>,
{
    type Item = T;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<Partition<T, F, O1, O>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(Partition::new(self.f, self.out1, input))
    }
}
