use std::cmp::Ordering;

use crate::lang::lattice::bottom::BottomRepr;
use crate::lang::lattice::{Compare, Convert, Debottom, Lattice, LatticeRepr, Merge, Top};
use crate::lang::tag;

pub struct Pair<La: Lattice, Lb: Lattice> {
    _phantom: std::marker::PhantomData<(La, Lb)>,
}
impl<La: Lattice, Lb: Lattice> Lattice for Pair<La, Lb> {}

pub struct PairRepr<Ra: LatticeRepr, Rb: LatticeRepr> {
    _phantom: std::marker::PhantomData<(Ra, Rb)>,
}
impl<Ra: LatticeRepr, Rb: LatticeRepr> LatticeRepr for PairRepr<Ra, Rb> {
    type Lattice = Pair<Ra::Lattice, Rb::Lattice>;
    type Repr = (Ra::Repr, Rb::Repr);
}

impl<SelfRA, SelfRB, DeltaRA, DeltaRB, La, Lb> Merge<PairRepr<DeltaRA, DeltaRB>>
    for PairRepr<SelfRA, SelfRB>
where
    La: Lattice,
    Lb: Lattice,
    SelfRA: LatticeRepr<Lattice = La>,
    SelfRB: LatticeRepr<Lattice = Lb>,
    DeltaRA: LatticeRepr<Lattice = La>,
    DeltaRB: LatticeRepr<Lattice = Lb>,
    SelfRA: Merge<DeltaRA>,
    SelfRB: Merge<DeltaRB>,
    DeltaRA: Convert<SelfRA>,
    DeltaRB: Convert<SelfRB>,
{
    fn merge(
        this: &mut <PairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr,
        delta: <PairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr,
    ) -> bool {
        // Do NOT use short-circuiting `&&`.
        SelfRA::merge(&mut this.0, delta.0) & SelfRB::merge(&mut this.1, delta.1)
    }
}

impl<SelfRA, SelfRB, DeltaRA, DeltaRB, La, Lb> Compare<PairRepr<DeltaRA, DeltaRB>>
    for PairRepr<SelfRA, SelfRB>
where
    La: Lattice,
    Lb: Lattice,
    SelfRA: LatticeRepr<Lattice = La>,
    SelfRB: LatticeRepr<Lattice = Lb>,
    DeltaRA: LatticeRepr<Lattice = La>,
    DeltaRB: LatticeRepr<Lattice = Lb>,
    SelfRA: Compare<DeltaRA>,
    SelfRB: Compare<DeltaRB>,
{
    fn compare(
        this: &<PairRepr<SelfRA, SelfRB> as LatticeRepr>::Repr,
        other: &<PairRepr<DeltaRA, DeltaRB> as LatticeRepr>::Repr,
    ) -> Option<Ordering> {
        let ord_a = SelfRA::compare(&this.0, &other.0);
        let ord_b = SelfRB::compare(&this.1, &other.1);
        if ord_a == ord_b {
            ord_a
        } else {
            None
        }
    }
}

impl<Ra: Debottom, Rb: Debottom> Debottom for PairRepr<Ra, Rb> {
    fn is_bottom(this: &Self::Repr) -> bool {
        Ra::is_bottom(&this.0) && Rb::is_bottom(&this.1)
    }

    type DebottomLr = PairRepr<BottomRepr<Ra::DebottomLr>, BottomRepr<Rb::DebottomLr>>;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        match (Ra::debottom(this.0), Rb::debottom(this.1)) {
            (None, None) => None,
            somes => Some(somes),
        }
    }
}

impl<Ra: Top, Rb: Top> Top for PairRepr<Ra, Rb> {
    fn is_top(this: &Self::Repr) -> bool {
        Ra::is_top(&this.0) && Rb::is_top(&this.1)
    }

    fn top() -> Self::Repr {
        (Ra::top(), Rb::top())
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    use super::set_union::SetUnionRepr;

    type HashSetHashSet =
        PairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashSetArraySet =
        PairRepr<SetUnionRepr<tag::HASH_SET, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type ArraySetHashSet =
        PairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::HASH_SET, u32>>;
    type ArraySetArraySet =
        PairRepr<SetUnionRepr<tag::ARRAY<8>, u32>, SetUnionRepr<tag::ARRAY<8>, u32>>;

    assert_impl_all!(
        HashSetHashSet: Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(
        HashSetArraySet: Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(
        ArraySetHashSet: Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );

    assert_not_impl_any!(
        ArraySetArraySet: Merge<HashSetHashSet>,
        Merge<HashSetArraySet>,
        Merge<ArraySetHashSet>,
        Merge<ArraySetArraySet>,
    );
}
