use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct FilterMap<O, F, In>
where
    O: Pusherator,
{
    out: O,
    f: F,
    _marker: PhantomData<fn(In)>,
}
impl<O, F, In> Pusherator for FilterMap<O, F, In>
where
    O: Pusherator,
    F: FnMut(In) -> Option<O::Item>,
{
    type Item = In;
    fn give(&mut self, item: Self::Item) {
        if let Some(item) = (self.f)(item) {
            self.out.give(item);
        }
    }
}
impl<O, F, In> FilterMap<O, F, In>
where
    O: Pusherator,
    F: FnMut(In) -> Option<O::Item>,
{
    pub fn new(f: F, out: O) -> Self {
        Self {
            out,
            f,
            _marker: PhantomData,
        }
    }
}

pub struct FilterMapBuild<P, F>
where
    P: PusheratorBuild,
{
    prev: P,
    f: F,
}
impl<P, F, Out> FilterMapBuild<P, F>
where
    P: PusheratorBuild,
    F: FnMut(P::Item) -> Option<Out>,
{
    pub fn new(prev: P, f: F) -> Self {
        Self {
            prev,
            f,
        }
    }
}
impl<P, F, Out> PusheratorBuild for FilterMapBuild<P, F>
where
    P: PusheratorBuild,
    F: FnMut(P::Item) -> Option<Out>,
{
    type Item = Out;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<FilterMap<O, F, P::Item>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(FilterMap::new(self.f, input))
    }
}
