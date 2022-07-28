use super::{Pusherator, PusheratorBuild};

use std::marker::PhantomData;

pub struct Filter<T, F, O>
where
    F: FnMut(&T) -> bool,
    O: Pusherator<Item = T>,
{
    out: O,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, F, O> Pusherator for Filter<T, F, O>
where
    F: FnMut(&T) -> bool,
    O: Pusherator<Item = T>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        if (self.f)(&item) {
            self.out.give(item);
        }
    }
}
impl<T, F, O> Filter<T, F, O>
where
    F: FnMut(&T) -> bool,
    O: Pusherator<Item = T>,
{
    pub fn new(f: F, out: O) -> Self {
        Self {
            out,
            f,
            _marker: PhantomData,
        }
    }
}

pub struct FilterBuild<T, F, P>
where
    F: FnMut(&T) -> bool,
    P: PusheratorBuild<Item = T>,
{
    prev: P,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, F, P> FilterBuild<T, F, P>
where
    F: FnMut(&T) -> bool,
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
impl<T, F, P> PusheratorBuild for FilterBuild<T, F, P>
where
    F: FnMut(&T) -> bool,
    P: PusheratorBuild<Item = T>,
{
    type Item = T;

    type Output<O: Pusherator<Item = Self::Item>> = P::Output<Filter<T, F, O>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        self.prev.build(Filter::new(self.f, input))
    }
}
