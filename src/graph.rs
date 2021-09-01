use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::tag;
use crate::lattice::{Lattice, LatticeRepr, Merge};
use crate::lattice::set_union::SetUnionRepr;
use crate::lattice::pair::PairRepr;
use crate::lattice::ord::MaxRepr;
use crate::lattice::bottom::BottomRepr;
use crate::hide::{Hide, Cumul, Delta};
use crate::op::{OpDelta, OpCumul};



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

pub struct DynSplitPoint<O: OpDelta + OpCumul> {
    _phantom: std::marker::PhantomData<O>,
}
impl<O: OpDelta + OpCumul> OpWrapper for DynSplitPoint<O> {
    type LatReprDeltaIn = O::LatReprDeltaIn;

    // Make the RHS OPTIONAL - via BottomRepr. -- RUNTIME PANIC IF None.
    // None is how represents a disconnected piece of the graph.
    type State = PairRepr<
        SetUnionRepr<tag::HASH_SET, Rc<dyn CompEdge<LatReprDeltaIn = O::LatReprDeltaOut>>> /* TODO: change equality check */,
        BottomRepr<O::State>
    >;

    #[must_use]
    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Feedback {
        let (state_ptrs, state_prec) = state.split_mut();

        // REVEAL?
        let state_prec = state_prec.reveal_ok_or_mut("Cannot run disconnected component!".to_owned())?;
        let element = O::get_delta(state_prec, element);

        for next_ptr in state_ptrs.reveal_ref() { // REVEAL!
            next_ptr.push(element.clone())?;
        }
        return Ok(());
    }
}



pub trait CompEdge {
    type LatReprDeltaIn: LatticeRepr;

    fn needs_cumul(&self) -> Hide<Cumul, MaxRepr<bool>>;

    fn push<'h>(&self, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>) -> Result<(), String>;
}

struct DynSplitCompEdge<O: OpWrapper> {
    state: RefCell<Hide<Cumul, O::State>>,
}
impl<O: OpWrapper> CompEdge for DynSplitCompEdge<O> {
    type LatReprDeltaIn = O::LatReprDeltaIn;

    fn needs_cumul(&self) -> Hide<Cumul, MaxRepr<bool>> {
        
    }

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

type GraphLatReprInternal = SetUnionRepr::<
    tag::HASH_SET,
    UniqueTag<OperatorId, Rc<dyn CompEdge<LatReprDeltaIn = MaxRepr<u32>>>> // TODO: <-- This lattice type should encompass the types of all egress.
>;

#[derive(Clone)]
pub struct GraphRepr {
    ops: <GraphLatReprInternal as LatticeRepr>::Repr,
}

impl Merge<GraphLatRepr> for GraphLatRepr {
    fn merge(this: &mut Self::Repr, delta: Self::Repr) -> bool {
        <GraphLatReprInternal as Merge<GraphLatReprInternal>>::merge(&mut this.ops, delta.ops)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::HashSet;
    use crate::op::identity::Identity;
    use crate::op::debug::Debug as DebugOp;

    #[test]
    fn test_basic() {
        type From_LatRepr = MaxRepr<u32>;
        type From_Pipeline = DebugOp<Identity<From_LatRepr>>;
        type From_WrapperEdge = DynSplitPoint<From_Pipeline>;

        type From_StateLatRepr = <From_WrapperEdge as OpWrapper>::State;

        let from_state: <From_StateLatRepr as LatticeRepr>::Repr = (HashSet::new(), Some(()));

        let from_ptr_edge_0: DynSplitCompEdge<From_WrapperEdge> = DynSplitCompEdge {
            state: RefCell::new(Hide::new(from_state)),
        };
        // Keep an RC around so we can push into it.
        // In the future we want the Graph to do this somehow.
        // Such as having the components generate their own inputs when you poke them?
        let from_ptr_edge_0 = Rc::new(from_ptr_edge_0);

        let tag_edge_to_0: UniqueTag<String, Rc<dyn CompEdge<LatReprDeltaIn = MaxRepr<u32>>>> = UniqueTag(
            "my_og_edge".to_owned(),
            from_ptr_edge_0.clone(),
        );

        let mut graph_state: <GraphLatReprInternal as LatticeRepr>::Repr = HashSet::new();
        graph_state.insert(tag_edge_to_0);


    }
}







#[derive(Debug, Clone, Copy)]
pub struct UniqueTag<T, U>(pub T, pub U);
impl<T: PartialEq, U> PartialEq for UniqueTag<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: Eq, U> Eq for UniqueTag<T, U> {}
impl<T: PartialOrd, U> PartialOrd for UniqueTag<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T: Ord, U> Ord for UniqueTag<T, U> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T: std::hash::Hash, U> std::hash::Hash for UniqueTag<T, U> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher
    {
        self.0.hash(state)
    }
}
impl<T, U> std::borrow::Borrow<T> for UniqueTag<T, U> {
    fn borrow(&self) -> &T {
        &self.0
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
