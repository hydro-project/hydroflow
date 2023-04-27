//! Pair compound lattice.
//!
//! Merge both nested lattices.

use std::cmp::Ordering;

use super::{Compare, ConvertFrom, Merge};

/// Dominating pair lattice.
///
/// `LatA` and `LatB` specify the nested lattice types.
pub struct Pair<LatA, LatB> {
    a: LatA,
    b: LatB,
}

impl<LatASelf, LatAOther, LatBSelf, LatBOther> Merge<Pair<LatAOther, LatBOther>>
    for Pair<LatASelf, LatBSelf>
where
    LatASelf: Merge<LatAOther>,
    LatBSelf: Merge<LatBOther>,
{
    fn merge(&mut self, other: Pair<LatAOther, LatBOther>) -> bool {
        // Do NOT use short-circuiting `||`.
        self.a.merge(other.a) | self.b.merge(other.b)
    }
}

impl<LatASelf, LatAOther, LatBSelf, LatBOther> ConvertFrom<Pair<LatAOther, LatBOther>>
    for Pair<LatASelf, LatBSelf>
where
    LatASelf: ConvertFrom<LatAOther>,
    LatBSelf: ConvertFrom<LatBOther>,
{
    fn from(other: Pair<LatAOther, LatBOther>) -> Self {
        Self {
            a: ConvertFrom::from(other.a),
            b: ConvertFrom::from(other.b),
        }
    }
}

impl<LatASelf, LatAOther, LatBSelf, LatBOther> Compare<Pair<LatAOther, LatBOther>>
    for Pair<LatASelf, LatBSelf>
where
    LatASelf: Compare<LatAOther>,
    LatBSelf: Compare<LatBOther>,
{
    fn compare(&self, other: &Pair<LatAOther, LatBOther>) -> Option<Ordering> {
        let ord_a = self.a.compare(&other.a);
        let ord_b = self.b.compare(&other.b);
        if ord_a == ord_b {
            ord_a
        } else {
            None
        }
    }
}

impl<LatA, LatB> Default for Pair<LatA, LatB>
where
    LatA: Default,
    LatB: Default,
{
    fn default() -> Self {
        let (a, b) = Default::default();
        Self { a, b }
    }
}
