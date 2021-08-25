// use std::borrow::Cow;
// use std::cell::RefCell;
// use crate::hide::{Hide, Delta, Cumul};
// use crate::lattice::LatticeRepr;
// use crate::lattice::null::NullRepr;
// use super::{Op, OpDelta};

// pub struct StaticIter<OLr: LatticeRepr, I: IntoIterator<Item = OLr::Repr>> {
//     _phantom: std::marker::PhantomData<(OLr, I)>,
// }

// impl<OLr: LatticeRepr, I: IntoIterator<Item = OLr::Repr>> Op for StaticIter<OLr, I> {
//     type ILatRepr = NullRepr;
//     type OLatRepr = OLr;

//     type State = SetUnionRepr<RefCell<I::IntoIter>>;
// }


// // impl<Lr: LatticeRepr, I: IntoIterator<Item = Lr::Repr>> OpDelta for StaticIter<Lr, I> {
// //     type Ord = IterOrder;

// //     fn poll_delta(&self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
// //         Poll::Ready(self.iter.borrow_mut().next().map(Hide::new))
// //     }
// // }

// impl<OLr: LatticeRepr, I: IntoIterator<Item = OLr::Repr>> OpDelta for StaticIter<OLr, I> {
//     fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::ILatRepr>>)
//         -> Cow<'h, Hide<Delta, Self::OLatRepr>>
//     {
//         let (prec_state, self_state) = state.split_mut();
//         let element = PrecOp::get_delta(prec_state, element);
//         Merge::merge_hide(self_state, element.clone().into_owned());
//         return Cow::Owned(Convert::convert_hide_cow(element)); // TODO
//     }
// }
