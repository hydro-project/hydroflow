use super::Pusherator;

use std::marker::PhantomData;

pub struct ForEach<T, F>
where
    F: FnMut(T),
{
    f: F,
    _marker: PhantomData<T>,
}
impl<T, F> Pusherator for ForEach<T, F>
where
    F: FnMut(T),
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        (self.f)(item)
    }
}
impl<T, F> ForEach<T, F>
where
    F: FnMut(T),
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}
