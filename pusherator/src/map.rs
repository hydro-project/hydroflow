use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct Map<T, U, F, O>
where
    F: FnMut(T) -> U,
    O: Pusherator<Item = U>,
{
    out: O,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, U, F, O> Pusherator for Map<T, U, F, O>
where
    F: FnMut(T) -> U,
    O: Pusherator<Item = U>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        self.out.give((self.f)(item));
    }
}
impl<T, U, F, O> Map<T, U, F, O>
where
    F: FnMut(T) -> U,
    O: Pusherator<Item = U>,
{
    pub fn new(f: F, out: O) -> Self {
        Self {
            out,
            f,
            _marker: PhantomData,
        }
    }
}

pub struct MapBuild<T, U, F, P>
where
    F: FnMut(T) -> U,
    P: PusheratorBuild<Item = T>,
{
    prev: P,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, U, F, P> MapBuild<T, U, F, P>
where
    F: FnMut(T) -> U,
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
impl<T, U, F, P> PusheratorBuild for MapBuild<T, U, F, P>
where
    F: FnMut(T) -> U,
    P: PusheratorBuild<Item = T>,
{
    type Item = U;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<Map<T, U, F, O>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(Map::new(self.f, input))
    }
}
