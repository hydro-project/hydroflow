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
    use crate::hide::{Hide};
    use crate::props::OpProps;

    use super::*;

    impl<R: LatticeRepr, Props: OpProps> Hide<BottomRepr<R>, Props> {
        pub fn reveal_unwrap(self) -> Hide<R, Props> {
            <Hide<R, Props>>::new(Self::into_reveal(self).unwrap())
        }
        pub fn reveal_unwrap_ref<'h>(&'h self) -> &'h Hide<R, Props> {
            <Hide<R, Props>>::ref_cast(Self::reveal_ref(self).as_ref().unwrap())
        }
        pub fn reveal_unwrap_mut<'h>(&'h mut self) -> &'h mut Hide<R, Props> {
            Hide::ref_cast_mut(self.reveal_mut().as_mut().unwrap())
        }

        pub fn reveal_ok_or<E>(self, err: E) -> Result<Hide<R, Props>, E> {
            Self::into_reveal(self).ok_or(err).map(|elem| <Hide<R, Props>>::new(elem))
        }
        pub fn reveal_ok_or_ref<'h, E>(&'h self, err: E) -> Result<&'h Hide<R, Props>, E> {
            Self::reveal_ref(self).as_ref().ok_or(err).map(|elem| <Hide<R, Props>>::ref_cast(elem))
        }
        pub fn reveal_ok_or_mut<'h, E>(&'h mut self, err: E) -> Result<&'h mut Hide<R, Props>, E> {
            self.reveal_mut().as_mut().ok_or(err).map(|elem| Hide::ref_cast_mut(elem))
        }
    }
}
