use std::cmp::Ordering;

use super::{Lattice, LatticeRepr, Merge, Compare, Convert, Debottom, Top};
use super::bottom::BottomRepr;

use crate::tag;

pub struct DomPair<La: Lattice, Lb: Lattice> {
    _phantom: std::marker::PhantomData<(La, Lb)>,
}
impl<La: Lattice, Lb: Lattice> Lattice for DomPair<La, Lb> {}

pub struct DomPairRepr<Ra: LatticeRepr, Rb: LatticeRepr> {
    _phantom: std::marker::PhantomData<(Ra, Rb)>,
}
impl<Ra: LatticeRepr, Rb: LatticeRepr> LatticeRepr for DomPairRepr<Ra, Rb> {
    type Lattice = DomPair<Ra::Lattice, Rb::Lattice>;
    type Repr = (Ra::Repr, Rb::Repr);
}


impl<SelfRA, SelfRB, DeltaRA, DeltaRB, La, Lb> Merge<DomPairRepr<DeltaRA, DeltaRB>> for DomPairRepr<SelfRA, SelfRB>
where
    La: Lattice,
    Lb: Lattice,
    SelfRA:  LatticeRepr<Lattice = La>,
    SelfRB:  LatticeRepr<Lattice = Lb>,
    DeltaRA: LatticeRepr<Lattice = La>,
    DeltaRB: LatticeRepr<Lattice = Lb>,
    SelfRA:  Merge<DeltaRA> + Compare<DeltaRA>,
    SelfRB:  Merge<DeltaRB> + Compare<DeltaRB>,
    DeltaRA: Convert<SelfRA>,
    DeltaRB: Convert<SelfRB>,
{
    fn merge(this: &mut <DomPairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr, delta: <DomPairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr) -> bool {
        match SelfRA::compare(&this.0, &delta.0) {
            None => {
                SelfRA::merge(&mut this.0, delta.0);
                SelfRB::merge(&mut this.1, delta.1);
                true
            }
            Some(Ordering::Equal) => {
                SelfRB::merge(&mut this.1, delta.1)
            }
            Some(Ordering::Less) => {
                *this = (
                    DeltaRA::convert(delta.0),
                    DeltaRB::convert(delta.1),
                );
                true
            }
            Some(Ordering::Greater) => false
        }
    }
}


impl<Ra: LatticeRepr, Rb: LatticeRepr> Convert<DomPairRepr<Ra, Rb>> for DomPairRepr<Ra, Rb> {
    fn convert(this: <DomPairRepr<Ra, Rb> as LatticeRepr>::Repr) -> <DomPairRepr<Ra, Rb> as LatticeRepr>::Repr {
        this
    }
}


impl<SelfRA, SelfRB, DeltaRA, DeltaRB, La, Lb> Compare<DomPairRepr<DeltaRA, DeltaRB>> for DomPairRepr<SelfRA, SelfRB>
where
    La: Lattice,
    Lb: Lattice,
    SelfRA:  LatticeRepr<Lattice = La>,
    SelfRB:  LatticeRepr<Lattice = Lb>,
    DeltaRA: LatticeRepr<Lattice = La>,
    DeltaRB: LatticeRepr<Lattice = Lb>,
    SelfRA:  Compare<DeltaRA>,
    SelfRB:  Compare<DeltaRB>,
{
    fn compare(this: &<DomPairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr, other: &<DomPairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr) -> Option<Ordering> {
        SelfRA::compare(&this.0, &other.0)
            .or_else(|| SelfRB::compare(&this.1, &other.1))
    }
}


impl<Ra: Debottom, Rb: Debottom> Debottom for DomPairRepr<Ra, Rb> {
    fn is_bottom(this: &Self::Repr) -> bool {
        Ra::is_bottom(&this.0) && Rb::is_bottom(&this.1)
    }

    type DebottomLr = DomPairRepr<BottomRepr<Ra::DebottomLr>, BottomRepr<Rb::DebottomLr>>;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        match (Ra::debottom(this.0), Rb::debottom(this.1)) {
            (None, None) => None,
            somes => Some(somes),
        }
    }
}

impl<Ra: Top, Rb: Top> Top for DomPairRepr<Ra, Rb> {
    fn is_top(this: &Self::Repr) -> bool {
        Ra::is_top(&this.0) && Rb::is_top(&this.1)
    }

    fn top() -> Self::Repr {
        (Ra::top(), Rb::top())
    }
}


fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    use super::set_union::{SetUnionRepr};

    type HashSetHashSet   = DomPairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashSetArraySet  = DomPairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type ArraySetHashSet  = DomPairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type ArraySetArraySet = DomPairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;

    assert_impl_all!(HashSetHashSet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(HashSetArraySet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(ArraySetHashSet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(ArraySetArraySet:
        Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );
}
