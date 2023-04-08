mod no_set_union;
mod set_union;

// pub use half_join_state_trait::HalfJoinState;
pub use no_set_union::HalfJoinStateNoSetUnion;
pub use set_union::HalfJoinStateSetUnion;

pub type JoinState<Key, V1, V2> = (
    HalfJoinStateSetUnion<Key, V1, V2>,
    HalfJoinStateSetUnion<Key, V2, V1>,
);

pub trait HalfJoinState<Key, ValBuild, ValProbe> {
    fn build(&mut self, k: Key, v: &ValBuild) -> bool;
    fn probe(&mut self, k: &Key, v: &ValProbe);
    fn pop_match(&mut self) -> Option<(Key, ValProbe, ValBuild)>;
}
