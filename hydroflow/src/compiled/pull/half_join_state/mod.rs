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
    /// Insert a key value pair into the join state, currently this is always inserting into a hash table
    /// If the key-value pair exists then it is implementation defined what hapepns, usually either two copies are stored or only one copy is stored.
    fn build(&mut self, k: Key, v: &ValBuild) -> bool;

    /// This function does the actual joining part of the join. It looks up a key in the local join state and creates matches
    /// The first match is return directly to the caller, and any additional matches are stored internally to be retrieved later with `pop_match`
    fn probe(&mut self, k: &Key, v: &ValProbe) -> Option<(Key, ValProbe, ValBuild)>;

    /// If there are any stored matches from previous calls to probe then this function will remove them one at a time and return it.
    fn pop_match(&mut self) -> Option<(Key, ValProbe, ValBuild)>;
}
