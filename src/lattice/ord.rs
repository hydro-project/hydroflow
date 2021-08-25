use super::*;

pub struct Max<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T: Ord> Lattice for Max<T> {}

pub struct MaxRepr<T: 'static + Ord> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Ord + Clone> LatticeRepr for MaxRepr<T> {
    type Lattice = Max<T>;
    type Repr = T;
}

impl<T: Ord + Clone> Merge<MaxRepr<T>> for MaxRepr<T> {
    fn merge(this: &mut <MaxRepr<T> as LatticeRepr>::Repr, delta: <MaxRepr<T> as LatticeRepr>::Repr) -> bool {
        if delta > *this {
            *this = delta;
            true
        }
        else {
            false
        }
    }
}

impl<T: Ord + Clone> Compare<MaxRepr<T>> for MaxRepr<T> {
    fn compare(this: &<MaxRepr<T> as LatticeRepr>::Repr, other: &<MaxRepr<T> as LatticeRepr>::Repr) -> Option<std::cmp::Ordering> {
        Some(this.cmp(other))
    }
}

impl<T: Ord + Clone> Convert<MaxRepr<T>> for MaxRepr<T> {
    fn convert(this: <MaxRepr<T> as LatticeRepr>::Repr) -> <MaxRepr<T> as LatticeRepr>::Repr {
        this
    }
}




pub struct Min<T: Ord> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T: Ord> Lattice for Min<T> {}

pub struct MinRepr<T: 'static + Ord> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Ord + Clone> LatticeRepr for MinRepr<T> {
    type Lattice = Min<T>;
    type Repr = T;
}

impl<T: Ord + Clone> Merge<MinRepr<T>> for MinRepr<T> {
    fn merge(this: &mut <MinRepr<T> as LatticeRepr>::Repr, delta: <MinRepr<T> as LatticeRepr>::Repr) -> bool {
        if delta < *this {
            *this = delta;
            true
        }
        else {
            false
        }
    }
}

impl<T: Ord + Clone> Compare<MinRepr<T>> for MinRepr<T> {
    fn compare(this: &<MinRepr<T> as LatticeRepr>::Repr, other: &<MinRepr<T> as LatticeRepr>::Repr) -> Option<std::cmp::Ordering> {
        Some(this.cmp(other).reverse())
    }
}

impl<T: Ord + Clone> Convert<MinRepr<T>> for MinRepr<T> {
    fn convert(this: <MinRepr<T> as LatticeRepr>::Repr) -> <MinRepr<T> as LatticeRepr>::Repr {
        this
    }
}



// TODO: use num traits for all variants of this.
impl Top for MaxRepr<u64> {
    fn is_top(this: &Self::Repr) -> bool {
        &u64::MAX == this
    }
    fn top() -> Self::Repr {
        u64::MAX
    }
}
