use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct Flatten<O, T>
where
    T: IntoIterator,
    O: Pusherator<Item = T::Item>,
{
    out: O,
    _marker: PhantomData<T>,
}
impl<O, T> Pusherator for Flatten<O, T>
where
    T: IntoIterator,
    O: Pusherator<Item = T::Item>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        for x in item {
            self.out.give(x);
        }
    }
}
impl<O, T> Flatten<O, T>
where
    T: IntoIterator,
    O: Pusherator<Item = T::Item>,
{
    pub fn new(out: O) -> Self {
        Self {
            out,
            _marker: PhantomData,
        }
    }
}

pub struct FlattenBuild<P>
where
    P: PusheratorBuild,
    P::Item: IntoIterator,
{
    prev: P,
}
impl<P> FlattenBuild<P>
where
    P: PusheratorBuild,
    P::Item: IntoIterator,
{
    pub fn new(prev: P) -> Self {
        Self { prev }
    }
}
impl<P> PusheratorBuild for FlattenBuild<P>
where
    P: PusheratorBuild,
    P::Item: IntoIterator,
{
    type Item = <P::Item as IntoIterator>::Item;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<Flatten<O, P::Item>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(Flatten::new(input))
    }
}
