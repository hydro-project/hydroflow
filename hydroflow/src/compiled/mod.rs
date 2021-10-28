use std::{collections::HashMap, marker::PhantomData};

pub mod pull;
use crate::lang::Lattice;

pub trait Pusherator: Sized {
    type Item;
    fn give(&mut self, item: Self::Item);

    fn map<F, T>(self, f: F) -> Map<T, Self::Item, F, Self>
    where
        F: Fn(T) -> Self::Item,
    {
        Map::new(f, self)
    }
}

pub struct Pivot<T, I, P>
where
    I: Iterator<Item = T>,
    P: Pusherator<Item = T>,
{
    pull: I,
    push: P,
}
impl<T, I, P> Pivot<T, I, P>
where
    I: Iterator<Item = T>,
    P: Pusherator<Item = T>,
{
    pub fn new(pull: I, push: P) -> Self {
        Self { pull, push }
    }

    pub fn step(&mut self) -> bool {
        if let Some(v) = self.pull.next() {
            self.push.give(v);
            true
        } else {
            false
        }
    }

    pub fn run(mut self) {
        for v in self.pull.by_ref() {
            self.push.give(v);
        }
    }
}

pub struct GroupBy<K, V, O>
where
    K: Eq + std::hash::Hash + Clone,
    V: Lattice + Clone,
    O: Pusherator<Item = (K, V)>,
{
    contents: HashMap<K, V>,
    out: O,
}
impl<K, V, O> Pusherator for GroupBy<K, V, O>
where
    K: Eq + std::hash::Hash + Clone,
    V: Lattice + Clone,
    O: Pusherator<Item = (K, V)>,
{
    type Item = (K, V);
    fn give(&mut self, item: Self::Item) {
        // TODO(justin): we need a more coherent understanding of time in order
        // to not emit a ton of extra stuff here.
        if let Some(v) = self.contents.get_mut(&item.0) {
            if v.join(&item.1) {
                self.out.give((item.0, v.clone()));
            }
        } else {
            self.contents.insert(item.0.clone(), item.1.clone());
            self.out.give(item)
        }
    }
}
impl<K, V, O> GroupBy<K, V, O>
where
    K: Eq + std::hash::Hash + Clone,
    V: Lattice + Clone,
    O: Pusherator<Item = (K, V)>,
{
    pub fn new(out: O) -> Self {
        GroupBy {
            contents: HashMap::new(),
            out,
        }
    }
}

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
        ForEach {
            f,
            _marker: PhantomData,
        }
    }
}

pub struct Map<T, U, F, O>
where
    F: Fn(T) -> U,
    O: Pusherator<Item = U>,
{
    out: O,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, U, F, O> Pusherator for Map<T, U, F, O>
where
    F: Fn(T) -> U,
    O: Pusherator<Item = U>,
{
    type Item = T;
    fn give(&mut self, item: Self::Item) {
        self.out.give((self.f)(item));
    }
}
impl<T, U, F, O> Map<T, U, F, O>
where
    F: Fn(T) -> U,
    O: Pusherator<Item = U>,
{
    pub fn new(f: F, out: O) -> Self {
        Map {
            out,
            f,
            _marker: PhantomData,
        }
    }
}

pub struct Filter<T, F, O>
where
    F: Fn(&T) -> bool,
    O: Pusherator<Item = T>,
{
    out: O,
    f: F,
    _marker: PhantomData<T>,
}
impl<T, F, O> Pusherator for Filter<T, F, O>
where
    F: Fn(&T) -> bool,
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
    F: Fn(&T) -> bool,
    O: Pusherator<Item = T>,
{
    pub fn new(f: F, out: O) -> Self {
        Filter {
            out,
            f,
            _marker: PhantomData,
        }
    }
}

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
        Partition {
            out1,
            out2,
            f,
            _marker: PhantomData,
        }
    }
}

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
        Tee {
            out1,
            out2,
            _marker: PhantomData,
        }
    }
}

pub struct TeeN<O, const N: usize>
where
    O: Pusherator,
    O::Item: Clone,
{
    outs: [O; N],
}
impl<O, const N: usize> Pusherator for TeeN<O, N>
where
    O: Pusherator,
    O::Item: Clone,
{
    type Item = O::Item;
    fn give(&mut self, item: Self::Item) {
        for out in &mut self.outs {
            out.give(item.clone()); // TODO: benchmark removing last extra clone.
        }
    }
}
impl<O, const N: usize> TeeN<O, N>
where
    O: Pusherator,
    O::Item: Clone,
{
    pub fn new(outs: [O; N]) -> Self {
        Self { outs }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        compiled::{Filter, ForEach, GroupBy, Map, Partition, Pivot, Pusherator, Tee},
        lang::Max,
    };

    #[test]
    fn linear_chains() {
        let mut v = Vec::new();
        let mut pusher = Map::new(
            |x| x * 2,
            Filter::new(|x| *x > 5, ForEach::new(|x| v.push(x))),
        );

        for i in 0..5 {
            pusher.give(i);
        }

        assert_eq!(v, vec![6, 8]);
    }

    #[test]
    fn partition() {
        let mut evens = Vec::new();
        let mut odds = Vec::new();
        let mut pusher = Partition::new(
            |x| x % 2 == 0,
            ForEach::new(|x| evens.push(x)),
            ForEach::new(|x| odds.push(x)),
        );

        for i in 0..5 {
            pusher.give(i);
        }

        assert_eq!(evens, vec![0, 2, 4]);
        assert_eq!(odds, vec![1, 3]);
    }

    #[test]
    fn tee() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut pusher = Tee::new(
            ForEach::new(|x| left.push(x)),
            ForEach::new(|x| right.push(x)),
        );

        for i in 0..5 {
            pusher.give(i);
        }

        assert_eq!(left, vec![0, 1, 2, 3, 4]);
        assert_eq!(right, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn tee_rcs() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut pusher = Map::new(
            Rc::new,
            Tee::new(
                ForEach::new(|x: Rc<i32>| left.push(*x)),
                ForEach::new(|x: Rc<i32>| right.push(*x)),
            ),
        );

        for i in 0..5 {
            pusher.give(i);
        }

        assert_eq!(left, vec![0, 1, 2, 3, 4]);
        assert_eq!(right, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn pivot() {
        let a = 0..10;
        let b = 10..20;

        let mut left = Vec::new();
        let mut right = Vec::new();

        let pivot = Pivot::new(
            a.into_iter().chain(b.into_iter()),
            Partition::new(
                |x| x % 2 == 0,
                ForEach::new(|x| left.push(x)),
                ForEach::new(|x| right.push(x)),
            ),
        );

        pivot.run();

        assert_eq!(left, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
        assert_eq!(right, vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19]);
    }

    #[test]
    fn map_union() {
        let mut v = Vec::new();
        let mut pusher = GroupBy::new(ForEach::new(|x| v.push(x)));

        pusher.give((1_usize, Max::new(1)));
        pusher.give((1, Max::new(2)));
        pusher.give((1, Max::new(1)));

        assert_eq!(v, vec![(1, Max::new(1)), (1, Max::new(2))]);
    }
}
