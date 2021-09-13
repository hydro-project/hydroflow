use std::borrow::Cow;
use ref_cast::RefCast;
use crate::lattice::LatticeRepr;
use crate::props::OpProps;

pub trait Qualifier {}
pub enum Delta {}
impl Qualifier for Delta {}
pub enum Cumul {}
impl Qualifier for Cumul {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<Lr: LatticeRepr + ?Sized, const PROPS: OpProps>
{
    value: Lr::Repr,
    // _phantom: std::marker::PhantomData,
}

pub const fn PROPS_into_delta(props: OpProps) -> OpProps {
    OpProps {
        lattice_ordered: true,
        ..props
    }
}

impl<Lr: LatticeRepr + ?Sized, const PROPS: OpProps> Hide<Lr, PROPS> {
    pub fn new(value: Lr::Repr) -> Self {
        Self {
            value,
            // _phantom: std::marker::PhantomData,
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

    pub fn into_delta(self) -> Hide<Lr, {PROPS_into_delta(PROPS)}> {
        Hide::new(self.value)
    }

    pub fn into_qualifier_reveal<const NEW_PROPS: OpProps>(self) -> Hide<Lr, NEW_PROPS> {
        Hide::new(self.value)
    }
}

impl<Lr: LatticeRepr, const PROPS: OpProps> Hide<Lr, PROPS> {
    pub fn reveal_cow<'h>(this: Cow<'h, Self>) -> Cow<'h, Lr::Repr> {
        match this {
            Cow::Owned(hide) => Cow::Owned(hide.into_reveal()),
            Cow::Borrowed(hide) => Cow::Borrowed(hide.reveal_ref()),
        }
    }

    pub fn as_delta_cow<'h>(this: Cow<'h, Self>) -> Cow<'h, Hide<Lr, {PROPS_into_delta(PROPS)}>> {
        match this {
            Cow::Owned(hide) => Cow::Owned(hide.into_delta()),
            Cow::Borrowed(hide) => Cow::Borrowed(Hide::ref_cast(hide.reveal_ref())),
        }
    }
}

impl<Lr: LatticeRepr, const PROPS: OpProps> Clone for Hide<Lr, PROPS> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            // _phantom: std::marker::PhantomData,
        }
    }
}
