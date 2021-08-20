use crate::lattice::LatticeRepr;

use ref_cast::RefCast;

pub trait Qualifier {}
pub enum Delta {}
impl Qualifier for Delta {}
pub enum Value {}
impl Qualifier for Value {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<'h, Y: Qualifier, Lr: LatticeRepr + ?Sized> {
    value: Lr::Repr,
    _phantom: std::marker::PhantomData<&'h Y>,
}

impl<'h, Y: Qualifier, Lr: LatticeRepr + ?Sized> Hide<'h, Y, Lr> {
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

    pub fn into_delta(self) -> Hide<'h, Delta, Lr> {
        Hide::new(self.value)
    }

    pub fn into_qualifier_reveal<Z: Qualifier>(self) -> Hide<'h, Z, Lr> {
        Hide::new(self.value)
    }
}

impl<'h, Y: Qualifier, Lr: LatticeRepr> Clone for Hide<'h, Y, Lr> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
