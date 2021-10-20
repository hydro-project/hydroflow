use std::cell::{RefCell};
use std::task::{Context, Poll, Waker};
use std::rc::{Rc};

use crate::lattice::LatticeRepr;
use crate::lattice::pair::PairRepr;
use crate::hide::{Hide, Qualifier, Delta, Value};

use super::*;

mod private {
    use super::*;

    pub struct SwitchOpState<R: LatticeRepr> {
        pub(super) waker: Option<Waker>,
        pub(super) delta: Option<Hide<Delta, R>>,
    }
    impl<R: LatticeRepr> Default for SwitchOpState<R> {
        fn default() -> Self {
            Self {
                waker: None,
                delta: None,
            }
        }
    }

    pub trait SwitchModePrivate<Ra: LatticeRepr, Rb: LatticeRepr> {
        type ThisLatRepr: LatticeRepr;
        type OtherLatRepr: LatticeRepr;

        const IS_A: bool;

        fn swap_state<'a>(a: &'a RefCell<SwitchOpState<Ra>>, b: &'a RefCell<SwitchOpState<Rb>>)
            -> (&'a RefCell<SwitchOpState<Self::ThisLatRepr>>, &'a RefCell<SwitchOpState<Self::OtherLatRepr>>);

        fn swap<Y: Qualifier>(a: Hide<Y, Ra>, b: Hide<Y, Rb>) -> (Hide<Y, Self::ThisLatRepr>, Hide<Y, Self::OtherLatRepr>);
    }

    impl<Ra: LatticeRepr, Rb: LatticeRepr> SwitchModePrivate<Ra, Rb> for SwitchModeA {
        type ThisLatRepr = Ra;
        type OtherLatRepr = Rb;

        const IS_A: bool = true;

        fn swap_state<'a>(a: &'a RefCell<SwitchOpState<Ra>>, b: &'a RefCell<SwitchOpState<Rb>>)
            -> (&'a RefCell<SwitchOpState<Self::ThisLatRepr>>, &'a RefCell<SwitchOpState<Self::OtherLatRepr>>)
        {
            (a, b)
        }

        fn swap<Y: Qualifier>(a: Hide<Y, Ra>, b: Hide<Y, Rb>) -> (Hide<Y, Self::ThisLatRepr>, Hide<Y, Self::OtherLatRepr>) {
            (a, b)
        }
    }

    impl<Ra: LatticeRepr, Rb: LatticeRepr> SwitchModePrivate<Ra, Rb> for SwitchModeB {
        type ThisLatRepr = Rb;
        type OtherLatRepr = Ra;

        const IS_A: bool = false;

        fn swap_state<'a>(a: &'a RefCell<SwitchOpState<Ra>>, b: &'a RefCell<SwitchOpState<Rb>>)
            -> (&'a RefCell<SwitchOpState<Self::ThisLatRepr>>, &'a RefCell<SwitchOpState<Self::OtherLatRepr>>)
        {
            (b, a)
        }

        fn swap<Y: Qualifier>(a: Hide<Y, Ra>, b: Hide<Y, Rb>) -> (Hide<Y, Self::ThisLatRepr>, Hide<Y, Self::OtherLatRepr>) {
            (b, a)
        }
    }
}

pub mod switch {
    use crate::lattice::LatticeRepr;
    use crate::metadata::Order;

    use super::private;

    pub trait SwitchMode<Ra: LatticeRepr, Rb: LatticeRepr>: private::SwitchModePrivate<Ra, Rb> {}

    pub enum SwitchModeA {}
    impl<Ra: LatticeRepr, Rb: LatticeRepr> SwitchMode<Ra, Rb> for SwitchModeA {}

    pub enum SwitchModeB {}
    impl<Ra: LatticeRepr, Rb: LatticeRepr> SwitchMode<Ra, Rb> for SwitchModeB {}

    pub struct SwitchOrder<O: Order, S>(std::marker::PhantomData<(O, S)>);
    impl<O: Order, S> Order for SwitchOrder<O, S> {}
}
use switch::*;



pub struct SwitchOp<O, Ra: LatticeRepr, Rb: LatticeRepr, S: SwitchMode<Ra, Rb>>
where
    O: Op<LatRepr = PairRepr<Ra, Rb>>,
{
    op: Rc<O>,
    state_a: Rc<RefCell<private::SwitchOpState<Ra>>>,
    state_b: Rc<RefCell<private::SwitchOpState<Rb>>>,
    _phantom: std::marker::PhantomData<S>,
}

impl<O, Ra: LatticeRepr, Rb: LatticeRepr> SwitchOp<O, Ra, Rb, SwitchModeA>
where
    O: Op<LatRepr = PairRepr<Ra, Rb>>,
{
    pub fn new(op: O) -> (Self, SwitchOp<O, Ra, Rb, SwitchModeB>) {
        let args = (Rc::new(op), Default::default());

        let op_a = {
            let (op, (state_a, state_b)) = (&args).clone();
            Self {
                op, state_a, state_b,
                _phantom: std::marker::PhantomData,
            }
        };

        let (op, (state_a, state_b)) = args;
        let op_b = SwitchOp {
            op, state_a, state_b,
            _phantom: std::marker::PhantomData,
        };

        (op_a, op_b)
    }
}

impl<O, Ra: LatticeRepr, Rb: LatticeRepr, S: SwitchMode<Ra, Rb>> Op for SwitchOp<O, Ra, Rb, S>
where
    O: Op<LatRepr = PairRepr<Ra, Rb>>,
{
    type LatRepr = S::ThisLatRepr;

    fn propegate_saturation(&self) {
        unimplemented!("TODO");
    }
}

impl<O: OpDelta, Ra: LatticeRepr, Rb: LatticeRepr, S: SwitchMode<Ra, Rb>> OpDelta for SwitchOp<O, Ra, Rb, S>
where
    O: OpDelta<LatRepr = PairRepr<Ra, Rb>>,
{
    type Ord = SwitchOrder<O::Ord, S>;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let (state_this, state_other) = S::swap_state(&self.state_a, &self.state_b);

        // Check if we have a value waiting.
        {
            let mut state_this = state_this.borrow_mut();
            match state_this.delta.take() {
                Some(polled) => {
                    return Poll::Ready(Some(polled));
                }
                None => {
                    state_this.waker.replace(ctx.waker().clone());
                }
            }
        }

        // Check if other splits are ready to receive a value.
        {
            let state_other = state_other.borrow();
            if let Some(_) = state_other.delta {
                // Other has it's value filled, wake it up and return pending.
                if let Some(waker) = &state_other.waker {
                    waker.wake_by_ref()
                }
                return Poll::Pending
            }
        }

        // Poll upstream.
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                let (delta_a, delta_b) = delta.split();
                let (delta_this, delta_other) = S::swap(delta_a, delta_b);

                {
                    // Handle other_state.
                    let mut state_other = state_other.borrow_mut();
                    let old_delta_opt = state_other.delta.replace(delta_other);
                    assert!(old_delta_opt.is_none());

                    if let Some(waker) = state_other.waker.take() {
                        waker.wake();
                    }
                }

                Poll::Ready(Some(delta_this))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}


impl<O: OpValue, Ra: LatticeRepr, Rb: LatticeRepr, S: SwitchMode<Ra, Rb>> OpValue for SwitchOp<O, Ra, Rb, S>
where
    O: OpDelta<LatRepr = PairRepr<Ra, Rb>>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        let (val_a, val_b) = self.op.get_value().split();
        S::swap(val_a, val_b).0
    }
}

