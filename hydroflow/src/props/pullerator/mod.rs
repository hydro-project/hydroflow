use crate::scheduled::context::Context;

use super::{Props, Spec};

pub trait PulleratorBuildBase {
    type ItemOut;
    type Build<'slf, 'ctx>: Iterator<Item = Self::ItemOut>;
}

pub trait PulleratorBuild: PulleratorBuildBase {
    type SpecOut: Spec;
    type PropsOut: Props;

    fn build<'slf, 'ctx>(&'slf mut self, context: &'ctx Context) -> Self::Build<'slf, 'ctx>;
}

// pub struct IteratorPullerator<Iter>
// where
//     Iter: IntoIterator,
// {
//     iter: Iter::IntoIter,
// }
// impl<Iter> IteratorPullerator<Iter>
// where
//     Iter: IntoIterator,
// {
//     pub fn new(iter: Iter) -> Self {
//         Self {
//             iter: iter.into_iter(),
//         }
//     }
// }
// impl<Iter> PulleratorBuild for IteratorPullerator<Iter>
// where
//     Iter: IntoIterator,
// {
//     type SpecOut = IteratorSpec<Iter>;
//     type PropsOut = (NonMonotonic, Duplicates);
//     type ItemOut = Iter::Item;

//     fn next(&mut self) -> Option<Self::ItemOut> {
//         self.iter.next()
//     }
// }

// pub struct IteratorSpec<Iter>(Iter)
// where
//     Iter: IntoIterator;
// impl<Iter> Spec for IteratorSpec<Iter> where Iter: IntoIterator {}

// pub struct ShufflePullerator<Prev>
// where
//     Prev: PulleratorBuild,
// {
//     pub prev: Prev,
// }
// impl<Prev> PulleratorBuild for ShufflePullerator<Prev>
// where
//     Prev: PulleratorBuild,
// {
//     type SpecOut = ShuffleSpec<Prev::SpecOut>;
//     type PropsOut = (NonMonotonic, <Prev::PropsOut as Props>::Duplicates);
//     type ItemOut = Prev::ItemOut;

//     fn next(&mut self) -> Option<Self::ItemOut> {
//         self.prev.next()
//     }
// }

// pub struct ShuffleSpec<PrevSpec>(PrevSpec)
// where
//     PrevSpec: Spec;
// impl<PrevSpec> Spec for ShuffleSpec<PrevSpec> where PrevSpec: Spec {}

// pub struct ShuffleReducePulleratorAxiom<Prev, InnerSpec>
// where
//     Prev: PulleratorBuild<SpecOut = ShuffleSpec<ShuffleSpec<InnerSpec>>>,
//     InnerSpec: Spec,
// {
//     pub prev: Prev,
// }
// impl<Prev, InnerSpec> PulleratorBuild for ShuffleReducePulleratorAxiom<Prev, InnerSpec>
// where
//     Prev: PulleratorBuild<SpecOut = ShuffleSpec<ShuffleSpec<InnerSpec>>>,
//     InnerSpec: Spec,
// {
//     type SpecOut = InnerSpec;
//     type PropsOut = Prev::PropsOut;
//     type ItemOut = Prev::ItemOut;

//     fn next(&mut self) -> Option<Self::ItemOut> {
//         self.prev.next()
//     }
// }

// #[cfg(test)]
// pub mod test {
//     use super::*;

//     #[test]
//     fn test_shuffle_reduce() {
//         let x = [1, 2, 3, 4];

//         // type X = impl Pullerator<SpecOut = IteratorSpec<[usize; 4]>>;

//         let prev = IteratorPullerator::new(x);
//         let prev = ShufflePullerator { prev };
//         let prev = ShufflePullerator { prev };
//         let prev = ShuffleReducePulleratorAxiom { prev };

//         fn assert_impl<X>(_: X)
//         where
//             X: PulleratorBuild<SpecOut = IteratorSpec<[usize; 4]>>,
//         {
//         }
//         assert_impl(prev);
//     }
// }
