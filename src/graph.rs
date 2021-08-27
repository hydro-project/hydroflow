use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::tag;
use crate::lattice::{Lattice, LatticeRepr, Merge};
use crate::lattice::set_union::SetUnionRepr;
use crate::lattice::pair::PairRepr;
use crate::lattice::null::NullRepr;
use crate::lattice::map_union::MapUnionRepr;
use crate::hide::{Hide, Cumul, Delta};
use crate::op::OpDelta;



// Status type.
type Feedback = Result<(), String>;


type OperatorId = String;


/// Specifies operator egress.
pub trait OpWrapper {
    type LatReprDeltaIn: LatticeRepr;

    type State: LatticeRepr;

    #[must_use]
    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Feedback;
}

pub struct DynSplitPoint<O: OpDelta> {
    _phantom: std::marker::PhantomData<O>,
}
impl<O: OpDelta> OpWrapper for DynSplitPoint<O> {
    type LatReprDeltaIn = O::LatReprDeltaIn;

    type State = PairRepr<SetUnionRepr<tag::HASH_SET, Rc<dyn OpPtr<LatReprDeltaIn = O::LatReprDeltaOut>>> /* TODO: change equality check */, O::State>;

    #[must_use]
    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Feedback {
        let (state_ptrs, state_prec) = state.split_mut();
        let element = O::get_delta(state_prec, element);

        let mut result = Ok(());
        for next_ptr in state_ptrs.reveal_ref() { // REVEAL!
            let next_result = next_ptr.push(element.clone());
            // Propegate error message. ?
            result = result.and(next_result);
        }
        return result;
    }
}



pub trait OpPtr {
    type LatReprDeltaIn: LatticeRepr;

    fn push<'h>(&self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Result<(), String>;
}

struct DynSplitOpPtr<O: OpWrapper> {
    state: Rc<RefCell<Hide<Cumul, O::State>>>,
}
impl<O: OpWrapper> OpPtr for DynSplitOpPtr<O> {
    type LatReprDeltaIn = O::LatReprDeltaIn;

    fn push<'h>(&self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Result<(), String> {
        O::get_delta(&mut *self.state.borrow_mut(), element)
    }
}


pub enum Graph {}
impl Lattice for Graph {}

pub enum GraphLatRepr {}
impl LatticeRepr for GraphLatRepr {
    type Lattice = Graph;
    type Repr = GraphRepr;
}

#[derive(Clone)]
pub struct GraphRepr {
    ops: HashMap<OperatorId, Rc<dyn OpPtr<LatReprDeltaIn = NullRepr>>>, // TODO: change equality check.
}

impl Merge<GraphLatRepr> for GraphLatRepr {
    fn merge(this: &mut Self::Repr, delta: Self::Repr) -> bool {
        <MapUnionRepr::<tag::HASH_MAP, _, _>>::merge(&mut this.ops, delta.ops)
    }
}












// type Full<T> = StaticRc<T, 3, 3>;
// type TwoThird<T> = StaticRc<T, 2, 3>;
// type OneThird<T> = StaticRc<T, 1, 3>;

// trait OpWrapperLatRepr {
//     type LatReprDeltaIn: LatticeRepr;

//     #[must_use]
//     fn push<'h>(&mut self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Result<(), String>;
// }

// struct OpWrapper<O: OpDelta> {
//     state: TwoThird<RefCell<Hide<Cumul, O::State>>>,
// }

// impl<O: OpDelta> OpWrapperLatRepr for OpWrapper<O> {
//     type LatReprDeltaIn = O::LatReprDeltaIn;

//     fn push<'h>(&mut self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Result<(), String> {
//         O::get_delta(&mut self.state, element);
//         Ok(())
//     }
// }











// enum StatusCode {
//     Yes,
//     No,
//     Yay,
// }

// trait OpDeltaIn {
//     type LatReprDeltaIn: LatticeRepr;

//     #[must_use]
//     fn get_delta<'h>(&mut self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
//         -> StatusCode;
// }

// #[repr(transparent)]
// struct MyOpDeltaIn<'s, O: OpDelta>
// where
//     O::LatReprDeltaOut: StatusCode,
// {
//     state: &'s mut Hide<Cumul, O::State>,
// }
// // TODO!!! Wait where TF are these stored? :P

// impl<'s, O: OpDelta> OpDeltaIn for MyOpDeltaIn<'s, O> {
//     // TODO: last op in chain has to return status
//     // i.e. be a dynamic split point
//     // or a network egress, for example.
//     type LatReprDeltaIn = O::LatReprDeltaIn;

//     #[must_use]
//     fn get_delta<'h>(self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
//         -> StatusCode
//     {
//         O::get_delta(self.state, element);
//         StatusCode::Yay
//     }
// }

// // trait AnyOpDelta {
// //     fn get_delta<'h>(&self, state: &'h mut dyn Any, element: Box<dyn Any>) -> Box<dyn Any>;

// //     fn id(&self) -> String;
// // }

// // pub struct OpDeltaWrapper<O: OpDelta> {
// //     _phantom: std::marker::PhantomData<O>,
// // }
// // impl<O: OpDelta> Default for OpDeltaWrapper<O> {
// //     fn default() -> Self {
// //         Self {
// //             _phantom: std::marker::PhantomData,
// //         }
// //     }
// // }

// // impl<O: OpDelta> AnyOpDelta for OpDeltaWrapper<O> {
// //     fn get_delta<'h>(&self, state: &'h mut dyn Any, element: Box<dyn Any>) -> Box<dyn Any> {
// //         let state = state.downcast_mut::<<O::State as LatticeRepr>::Repr>()
// //             .expect("AnyOpDelta received wrong state type.");

// //         let element = element.downcast_ref::<<O::LatReprDeltaIn as LatticeRepr>::Repr>()
// //             .expect("AnyOpDelta received wrong element type.");

// //         let element = O::get_delta(Hide::ref_cast_mut(state), Cow::Borrowed(Hide::ref_cast(element)));
// //         return Box::new(element.into_owned().into_reveal());
// //     }
// // }

// // struct Edge<A, B>
// // where
// //     A: OpDelta,
// //     B: OpDelta<LatReprDeltaIn = A::LatReprDeltaOut>,
// // {
// //     _phantom: std::marker::PhantomData<(A, B)>,
// //     a: String,
// //     b: String,
// // }

// // trait AnyEdge {}
// // impl<A, B> AnyEdge for Edge<A, B>
// // where
// //     A: OpDelta,
// //     B: OpDelta<LatReprDeltaIn = A::LatReprDeltaOut>,
// // {}



// // use std::rc::Rc;
// // pub struct Graph {
// //     components: Vec<Rc<dyn AnyOpDelta>>,
// //     edges: Vec<(Rc<dyn AnyOpDelta>, Rc<dyn AnyOpDelta>)>,
// // }

// // // pub trait Run {
// // //     fn run(&mut self);
// // // }

// // // pub struct Edge<A, B>
// // // where
// // //     A: OpDelta,
// // //     B: OpDelta<LatReprDeltaIn = A::LatReprDeltaOut>,
// // // {
// // //     a: A,
// // //     b: B,
// // // }

// // // impl<A, B> Run for Edge<A, B>
// // // where
// // //     A: OpDelta,
// // //     B: OpDelta<LatReprDeltaIn = A::LatReprDeltaOut>,
// // // {
// // //     fn run(&mut self) {

// // //     }
// // // }
