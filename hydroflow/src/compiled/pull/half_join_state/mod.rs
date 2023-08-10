mod fold;
mod fold_from;
mod multiset;
mod multiset2;
mod reduce;
mod set;

use std::collections::hash_map::Iter;
use std::slice;

pub use fold::HalfJoinStateFold;
pub use fold_from::HalfJoinStateFoldFrom;
pub use multiset::HalfMultisetJoinState;
pub use multiset2::HalfJoinStateMultiset;
pub use reduce::HalfJoinStateReduce;
pub use set::HalfSetJoinState;
use smallvec::SmallVec;

pub trait HalfJoinState<Key, ValBuild, ValProbe> {
    fn build(&mut self, k: Key, v: &ValBuild) -> bool;
    fn probe(&mut self, k: &Key, v: &ValProbe);
    fn pop_match(&mut self) -> Option<(Key, ValProbe, ValBuild)>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter(&self) -> Iter<'_, Key, SmallVec<[ValBuild; 1]>>;
    fn full_probe(&self, k: &Key) -> slice::Iter<'_, ValBuild>;
}
