use ref_cast::RefCast;
use crate::lattice::LatticeRepr;


pub trait Qualifier {}
pub enum Delta {}
impl Qualifier for Delta {}
pub enum Cumul {}
impl Qualifier for Cumul {}

#[derive(RefCast)]
#[repr(transparent)]
pub struct Hide<Y: Qualifier, Lr: LatticeRepr + ?Sized>
{
    value: Lr::Repr,
    _phantom: std::marker::PhantomData<Y>,
}

impl<Y: Qualifier, Lr: LatticeRepr + ?Sized> Hide<Y, Lr> {
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

    pub fn into_delta(self) -> Hide<Delta, Lr> {
        Hide::new(self.value)
    }

    pub fn into_qualifier_reveal<Z: Qualifier>(self) -> Hide<Z, Lr> {
        Hide::new(self.value)
    }
}

impl<'h, Y: Qualifier, Lr: LatticeRepr + 'h> Clone for Hide<Y, Lr> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
