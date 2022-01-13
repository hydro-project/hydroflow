use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct FlatMap<T, U, F, O>
where
    F: FnMut(T) -> U,
    U: IntoIterator,
    O: Pusherator<Item = U::Item>,
{
    out: O,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, U, F, O> Pusherator for FlatMap<T, U, F, O>
where
    F: FnMut(T) -> U,
    U: IntoIterator,
    O: Pusherator<Item = U::Item>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        for x in (self.f)(item) {
            self.out.give(x);
        }
    }
}
impl<T, U, F, O> FlatMap<T, U, F, O>
where
    F: FnMut(T) -> U,
    U: IntoIterator,
    O: Pusherator<Item = U::Item>,
{
    pub fn new(f: F, out: O) -> Self {
        Self {
            out,
            f,
            _marker: PhantomData,
        }
    }
}

pub struct FlatMapBuild<T, U, F, P>
where
    F: FnMut(T) -> U,
    U: IntoIterator,
    P: PusheratorBuild<Item = T>,
{
    prev: P,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, U, F, P> FlatMapBuild<T, U, F, P>
where
    F: FnMut(T) -> U,
    U: IntoIterator,
    P: PusheratorBuild<Item = T>,
{
    pub fn new(prev: P, f: F) -> Self {
        Self {
            prev,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, U, F, P> PusheratorBuild for FlatMapBuild<T, U, F, P>
where
    F: FnMut(T) -> U,
    U: IntoIterator,
    P: PusheratorBuild<Item = T>,
{
    type Item = U::Item;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<FlatMap<T, U, F, O>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(FlatMap::new(self.f, input))
    }
}
