//! Hydroflow lattice and flow properties, very WIP.
#![allow(missing_docs)]

pub mod wrap;

pub trait Spec {}

pub trait Props {
    type Monotonicity: PropMonotonicity;
    type Duplicates: PropDuplicates;
}
impl<Monotonicity, Duplicates> Props for (Monotonicity, Duplicates)
where
    Monotonicity: PropMonotonicity,
    Duplicates: PropDuplicates,
{
    type Monotonicity = Monotonicity;
    type Duplicates = Duplicates;
}

pub trait PropMonotonicity {}
pub struct NonMonotonic;
impl PropMonotonicity for NonMonotonic {}
pub struct Monotonic;
impl PropMonotonicity for Monotonic {}
pub struct Consecutive;
impl PropMonotonicity for Consecutive {}

pub trait PropDuplicates {}
pub struct Duplicates;
impl PropDuplicates for Duplicates {}
pub struct NoDuplicates;
impl PropDuplicates for NoDuplicates {}
