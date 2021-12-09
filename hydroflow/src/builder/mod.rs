use crate::compiled::{Filter, ForEach, Map, Pusherator, Tee};
use std::marker::PhantomData;

pub trait PusheratorBuild {
    type Item;

    type Output<O: Pusherator<Item = Self::Item>>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>;

    fn map<U, F>(self, f: F) -> MapBuild<Self::Item, U, F, Self>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> U,
    {
        MapBuild {
            prev: self,
            f,
            _marker: PhantomData,
        }
    }

    fn filter<F>(self, f: F) -> FilterBuild<Self::Item, F, Self>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        FilterBuild {
            prev: self,
            f,
            _marker: PhantomData,
        }
    }

    fn tee<O1>(self, first_out: O1) -> TeeBuild<Self::Item, O1, Self>
    where
        Self: Sized,
        Self::Item: Clone,
        O1: Pusherator<Item = Self::Item>,
    {
        TeeBuild {
            prev: self,
            first_out,
            _marker: PhantomData,
        }
    }

    fn for_each<F>(self, f: F) -> Self::Output<ForEach<Self::Item, F>>
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        self.build(ForEach::new(f))
    }
}

pub struct InputBuild<T>(PhantomData<T>);
impl<T> Default for InputBuild<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> InputBuild<T> {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<T> PusheratorBuild for InputBuild<T> {
    type Item = T;

    type Output<O: Pusherator<Item = Self::Item>> = O;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        input
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

pub struct FilterBuild<T, F, P>
where
    F: FnMut(&T) -> bool,
    P: PusheratorBuild<Item = T>,
{
    prev: P,
    f: F,
    _marker: PhantomData<T>,
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

pub trait IteratorToPusherator: Iterator {
    fn pusherator(self) -> BuiltSubgraphBuild<Self>
    where
        Self: Sized,
    {
        BuiltSubgraphBuild { pull: self }
    }
}
impl<I> IteratorToPusherator for I where I: Sized + Iterator {}

pub struct BuiltSubgraph<I, O>
where
    I: Iterator,
    O: Pusherator<Item = I::Item>,
{
    pull: I,
    push: O,
}
impl<I, O> BuiltSubgraph<I, O>
where
    I: Iterator,
    O: Pusherator<Item = I::Item>,
{
    pub fn run_no_context(mut self) {
        for item in self.pull {
            self.push.give(item);
        }
    }
}
impl<I, O> crate::scheduled::subgraph::Subgraph for BuiltSubgraph<I, O>
where
    I: Iterator,
    O: Pusherator<Item = I::Item>,
{
    fn run(&mut self, _context: crate::scheduled::Context<'_>) {
        // TODO(mingwei): something smart with context.
        // TODO(mingwei): how does this handle statefulness/reentrancy? Do we need to rebuild it every time, or something?
        for item in &mut self.pull {
            self.push.give(item);
        }
    }
}

pub struct BuiltSubgraphBuild<I>
where
    I: Iterator,
{
    pull: I,
}
impl<I> PusheratorBuild for BuiltSubgraphBuild<I>
where
    I: Iterator,
{
    type Item = I::Item;

    type Output<O: Pusherator<Item = Self::Item>> = BuiltSubgraph<I, O>;
    fn build<O>(self, input: O) -> Self::Output<O>
    where
        O: Pusherator<Item = Self::Item>,
    {
        BuiltSubgraph {
            pull: self.pull,
            push: input,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_constructed() {
        let pb = InputBuild::<usize>(PhantomData);
        let pb = FilterBuild {
            prev: pb,
            f: |&x| 0 == x % 2,
            _marker: PhantomData,
        };
        let pb = MapBuild {
            prev: pb,
            f: |x| x * x,
            _marker: PhantomData,
        };

        let mut output = Vec::new();
        let mut pusherator = pb.build(ForEach::new(|x| output.push(x)));

        for x in 0..10 {
            pusherator.give(x);
        }

        assert_eq!(&[0, 4, 16, 36, 64], &*output);
    }

    #[test]
    fn test_builder() {
        let mut output = Vec::new();

        let mut pusherator = <InputBuild<usize>>::new()
            .filter(|&x| 0 == x % 2)
            .map(|x| x * x)
            .for_each(|x| output.push(x));

        for x in 0..10 {
            pusherator.give(x);
        }

        assert_eq!(&[0, 4, 16, 36, 64], &*output);
    }

    #[test]
    fn test_builder_tee() {
        let mut output_evn = Vec::new();
        let mut output_odd = Vec::new();

        let mut pusherator = <InputBuild<usize>>::new()
            .tee(
                <InputBuild<usize>>::new()
                    .filter(|&x| 0 == x % 2)
                    .for_each(|x| output_evn.push(x)),
            )
            .filter(|&x| 1 == x % 2)
            .for_each(|x| output_odd.push(x));

        for x in 0..10 {
            pusherator.give(x);
        }

        assert_eq!(&[0, 2, 4, 6, 8], &*output_evn);
        assert_eq!(&[1, 3, 5, 7, 9], &*output_odd);
    }

    #[test]
    fn test_built_subgraph() {
        let mut output_evn = Vec::new();
        let mut output_odd = Vec::new();

        let built_subgraph = [1, 2, 3, 4, 5]
            .into_iter()
            .chain([3, 4, 5, 6, 7])
            .map(|x| x * 9)
            .pusherator()
            .map(|x| if 0 == x % 2 { x / 2 } else { 3 * x + 1 })
            .tee(
                <InputBuild<usize>>::new()
                    .filter(|&x| 0 == x % 2)
                    .for_each(|x| output_evn.push(x)),
            )
            .filter(|&x| 1 == x % 2)
            .for_each(|x| output_odd.push(x));

        built_subgraph.run_no_context();

        assert_eq!(&[28, 82, 18, 136, 82, 18, 136, 190], &*output_evn);
        assert_eq!(&[9, 27], &*output_odd);
    }
}
