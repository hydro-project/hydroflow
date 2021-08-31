use super::{LatticeRepr, Debottom};

pub struct BottomRepr<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> LatticeRepr for BottomRepr<Lr> {
    type Lattice = Lr::Lattice;
    type Repr = Option<Lr::Repr>;
}

impl<Lr: LatticeRepr> Debottom for BottomRepr<Lr> {
    fn is_bottom(this: &Self::Repr) -> bool {
        this.is_none()
    }

    type DebottomLr = Lr;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        this
    }
}

mod fns {
    use ref_cast::RefCast;
    use crate::hide::{Hide, Qualifier};

    use super::*;

    impl<Y: Qualifier, R: LatticeRepr> Hide<Y, BottomRepr<R>> {
        pub fn unwrap(self) -> Hide<Y, R> {
            Hide::new(self.into_reveal().unwrap())
        }
        pub fn unwrap_ref<'h>(&'h self) -> &'h Hide<Y, R> {
            Hide::ref_cast(self.reveal_ref().as_ref().unwrap())
        }
        pub fn unwrap_mut<'h>(&'h mut self) -> &'h mut Hide<Y, R> {
            Hide::ref_cast_mut(self.reveal_mut().as_mut().unwrap())
        }
    }
}
