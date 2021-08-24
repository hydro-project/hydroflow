use std::borrow::Cow;
use ref_cast::RefCast;
use crate::lattice::LatticeRepr;


pub trait Qualifier {}
pub enum Delta {}
impl Qualifier for Delta {}
pub enum Cumul {}
impl Qualifier for Cumul {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<'h, Y: Qualifier, Lr: LatticeRepr + ?Sized + 'h>
{
    value: Cow<'h, Lr::Repr>,
    _phantom: std::marker::PhantomData<Y>,
}

impl<'h, Y: Qualifier, Lr: LatticeRepr + ?Sized + 'h> Hide<'h, Y, Lr> {
    pub fn new(value: Lr::Repr) -> Self {
        Self::from_cow(Cow::Owned(value))
    }
    pub fn from_cow(value: Cow<'h, Lr::Repr>) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn into_reveal(self) -> Lr::Repr {
        self.value.into_owned()
    }

    pub fn reveal_ref(&self) -> &Lr::Repr {
        &self.value
    }

    pub fn reveal_mut(&mut self) -> &mut Lr::Repr {
        self.value.to_mut()
    }

    pub fn into_delta(self) -> Hide<'h, Delta, Lr> {
        Hide::from_cow(self.value)
    }

    pub fn into_qualifier_reveal<Z: Qualifier>(self) -> Hide<'h, Z, Lr> {
        Hide::from_cow(self.value)
    }
}

impl<'h, Y: Qualifier, Lr: LatticeRepr + 'h> Clone for Hide<'h, Y, Lr> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
