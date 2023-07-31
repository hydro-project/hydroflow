mod fold;
mod fold_from;
mod multiset;
mod multiset2;
mod reduce;
mod set;

pub use fold::HalfJoinStateFold;
pub use fold_from::HalfJoinStateFoldFrom;
pub use multiset::HalfMultisetJoinState;
pub use multiset2::HalfJoinStateMultiset;
pub use reduce::HalfJoinStateReduce;
pub use set::HalfSetJoinState;

pub type SetJoinState<Key, V1, V2> = (HalfSetJoinState<Key, V1, V2>, HalfSetJoinState<Key, V2, V1>);

pub trait HalfJoinState<Key, ValBuild, ValProbe> {
    fn build(&mut self, k: Key, v: &ValBuild) -> bool;
    fn probe(&mut self, k: &Key, v: &ValProbe);
    fn pop_match(&mut self) -> Option<(Key, ValProbe, ValBuild)>;
}
