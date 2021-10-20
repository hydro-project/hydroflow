use std::cell::Cell;
use std::task::{Context, Poll};

use crate::hide::{Delta, Hide};
use crate::lattice::set_union::{SetUnion, SetUnionRepr};
use crate::lattice::LatticeRepr;
use crate::tag;

use super::*;

pub struct ZipOp<A: OpDelta, B: OpDelta<Ord = A::Ord>, T: Clone, U: Clone>
where
    A::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<U>>,
    <A::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <B::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = U>,
{
    op_a: A,
    op_b: B,
    delta_a_opt: Cell<Option<Hide<Delta, A::LatRepr>>>,
}

impl<A: OpDelta, B: OpDelta<Ord = A::Ord>, T: Clone, U: Clone> ZipOp<A, B, T, U>
where
    A::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<U>>,
    <A::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <B::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = U>,
{
    pub fn new(op_a: A, op_b: B) -> Self {
        Self {
            op_a,
            op_b,
            delta_a_opt: Cell::new(None),
        }
    }
}

impl<A: OpDelta, B: OpDelta<Ord = A::Ord>, T: Clone, U: Clone> Op for ZipOp<A, B, T, U>
where
    A::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<U>>,
    <A::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <B::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = U>,
{
    type LatRepr = SetUnionRepr<tag::VEC, (T, U)>;

    fn propegate_saturation(&self) {
        self.op_a.propegate_saturation();
        self.op_b.propegate_saturation()
    }
}

impl<A: OpDelta, B: OpDelta<Ord = A::Ord>, T: Clone, U: Clone> OpDelta for ZipOp<A, B, T, U>
where
    A::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<U>>,
    <A::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <B::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = U>,
{
    type Ord = A::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let delta_a_opt = self.delta_a_opt.take();

        let delta_a = match delta_a_opt {
            Some(delta_a) => delta_a,
            None => match self.op_a.poll_delta(ctx) {
                Poll::Ready(Some(delta_a)) => delta_a,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            },
        };

        match self.op_b.poll_delta(ctx) {
            Poll::Ready(Some(delta_b)) => {
                let mut out = Vec::new();
                let delta_as: Vec<_> = delta_a.into_reveal().into_iter().collect();
                let delta_bs: Vec<_> = delta_b.into_reveal().into_iter().collect();
                for val_a in delta_as.iter() {
                    for val_b in delta_bs.iter() {
                        out.push((val_a.clone(), val_b.clone()));
                    }
                }
                return Poll::Ready(Some(Hide::new(out)));
            }
            Poll::Ready(None) => panic!(),
            Poll::Pending => {}
        };

        self.delta_a_opt.replace(Some(delta_a));

        Poll::Pending
    }
}
