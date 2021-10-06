use std::borrow::Cow;
use ref_cast::RefCast;
use crate::lattice::LatticeRepr;
use crate::props::{OpProps};

pub trait Qualifier {}
pub enum Delta {}
impl Qualifier for Delta {}
pub enum Cumul {}
impl Qualifier for Cumul {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<Lr: LatticeRepr + ?Sized, Props: OpProps>
{
    value: Lr::Repr,
    _phantom: std::marker::PhantomData<Props>,
}

impl<Lr: LatticeRepr + ?Sized, Props: OpProps> Hide<Lr, Props> {
    pub fn new(value: Lr::Repr) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn into_reveal(self) -> Lr::Repr {
        self.value
    }

    pub fn reveal_ref(&self) -> &Lr::Repr {
        &self.value
    }

    pub fn reveal_mut(&mut self) -> &mut Lr::Repr {
        &mut self.value
    }

    // pub fn into_delta(self) -> Hide<Lr, OpProps_Or<Props, (False, False, True)>>
    // where
    //     Props: OpProps_OrComp<(False, False, True)>, // TODO THIS IS WRONG
    // {
    //     Hide::<Lr, OpProps_Or<Props, (False, False, True)>>::new(self.value)
    // }

    pub fn into_qualifier_reveal<NewProps: OpProps>(self) -> Hide<Lr, NewProps> {
        Hide::<Lr, NewProps>::new(self.value)
    }
}

impl<Lr: LatticeRepr, Props: OpProps> Hide<Lr, Props> {
    pub fn reveal_cow<'h>(this: Cow<'h, Self>) -> Cow<'h, Lr::Repr> {
        match this {
            Cow::Owned(hide) => Cow::Owned(Self::into_reveal(hide)),
            Cow::Borrowed(hide) => Cow::Borrowed(Self::reveal_ref(hide)),
        }
    }

    // pub fn as_delta_cow<'h>(this: Cow<'h, Self>) -> Cow<'h, Hide<Lr, OpProps_Or<Props, (False, False, True)>>>
    // where
    //     Props: OpProps_OrComp<(False, False, True)>, // TODO THIS IS WRONG
    // {
    //     match this {
    //         Cow::Owned(hide) => Cow::Owned(Self::into_delta(hide)),
    //         Cow::Borrowed(hide) => Cow::Borrowed(Hide::<Lr, OpProps_Or<Props, (False, False, True)>>::ref_cast(Self::reveal_ref(hide))),
    //     }
    // }
}

impl<Lr: LatticeRepr, Props: OpProps> Clone for Hide<Lr, Props> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
