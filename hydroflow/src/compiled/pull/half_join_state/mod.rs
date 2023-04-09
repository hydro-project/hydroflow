mod multiset;
mod set;

// pub use half_join_state_trait::HalfJoinState;
pub use multiset::HalfMultisetJoinState;
pub use set::HalfSetJoinState;

pub type SetJoinState<Key, V1, V2> = (HalfSetJoinState<Key, V1, V2>, HalfSetJoinState<Key, V2, V1>);

pub trait HalfJoinState<Key, ValBuild, ValProbe> {
    fn build(&mut self, k: Key, v: &ValBuild) -> bool;
    fn probe(&mut self, k: &Key, v: &ValProbe);
    fn pop_match(&mut self) -> Option<(Key, ValProbe, ValBuild)>;
}
