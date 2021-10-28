// use std::collections::HashMap;

// TODO(justin): quick and dirty lattice implementation, we probably want to
// copy over spinachflow's.

pub trait Lattice {
    // Returns whether it changed or not.
    fn join(&mut self, other: &Self) -> bool;
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Max<T>
where
    T: Ord + Clone + Eq,
{
    t: T,
}
impl<T> Max<T>
where
    T: Ord + Clone,
{
    pub fn new(t: T) -> Self {
        Max { t }
    }
}
impl<T> Lattice for Max<T>
where
    T: Ord + Clone,
{
    fn join(&mut self, other: &Self) -> bool {
        if self.t < other.t {
            self.t = other.t.clone();
            true
        } else {
            false
        }
    }
}
