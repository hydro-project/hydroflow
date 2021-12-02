// TODO(justin): For now, require these to be totally ordered. We should make
// them lattices later, though.
pub trait Timestamp: Clone + Ord + std::fmt::Debug {
    fn min() -> Self;
    fn max() -> Self;
    fn le(&self, other: &Self) -> bool;
    fn join(&self, other: &Self) -> Self;
    fn meet(&self, other: &Self) -> Self;
}

impl Timestamp for usize {
    fn min() -> Self {
        usize::MIN
    }

    fn max() -> Self {
        usize::MAX
    }

    fn le(&self, other: &usize) -> bool {
        *self <= *other
    }

    fn join(&self, other: &Self) -> Self {
        std::cmp::max(*self, *other)
    }

    fn meet(&self, other: &Self) -> Self {
        std::cmp::min(*self, *other)
    }
}

impl Timestamp for () {
    fn min() -> Self {}
    fn max() -> Self {}

    fn le(&self, _: &()) -> bool {
        true
    }

    fn join(&self, _: &()) -> Self {}
    fn meet(&self, _: &()) -> Self {}
}

// TODO(justin): for partially ordered time this wants to be an antichain.
// Represents some boundary of time. Typically used by an operator to signal
// that it has no more data prior to a particular point in time.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Frontier<T> {
    v: Option<T>,
}

impl<T: Timestamp> Frontier<T> {
    pub fn new(v: Option<T>) -> Self {
        Frontier { v }
    }

    pub fn get(&self) -> Option<T> {
        self.v.clone()
    }

    pub fn join_in(&mut self, t: &T) {
        self.v = Some(match &mut self.v {
            Some(v) => v.join(t),
            None => t.clone(),
        })
    }

    pub fn join_with(&mut self, t: &Self) {
        if let Some(t) = &t.v {
            self.join_in(t);
        }
    }

    pub fn meet(&mut self, t: &Self) -> Self {
        Frontier::new(match (&self.v, &t.v) {
            (Some(u), Some(v)) => Some(u.meet(v)),
            _ => None,
        })
    }

    pub fn dominates(&self, t: &T) -> bool {
        match &self.v {
            Some(v) => v >= t,
            None => false,
        }
    }
}
