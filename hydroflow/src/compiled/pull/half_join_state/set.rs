use crate::lang::clear::Clear;

use super::HalfJoinState;
use std::collections::{hash_map::Entry, VecDeque};

type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

#[derive(Debug)]
pub struct HalfSetJoinState<Key, ValBuild, ValProbe> {
    /// Table to probe, vec val contains all matches.
    table: HashMap<Key, Vec<ValBuild>>,
    /// Not-yet emitted matches.
    current_matches: VecDeque<(Key, ValProbe, ValBuild)>,
}
impl<Key, ValBuild, ValProbe> Default for HalfSetJoinState<Key, ValBuild, ValProbe> {
    fn default() -> Self {
        Self {
            table: HashMap::default(),
            current_matches: VecDeque::default(),
        }
    }
}
impl<Key, ValBuild, ValProbe> Clear for HalfSetJoinState<Key, ValBuild, ValProbe> {
    fn clear(&mut self) {
        self.table.clear();
        self.current_matches.clear();
    }
}
impl<Key, ValBuild, ValProbe> HalfJoinState<Key, ValBuild, ValProbe>
    for HalfSetJoinState<Key, ValBuild, ValProbe>
where
    Key: Clone + Eq + std::hash::Hash,
    ValBuild: Clone + Eq,
    ValProbe: Clone,
{
    fn build(&mut self, k: Key, v: &ValBuild) -> bool {
        let entry = self.table.entry(k);

        match entry {
            Entry::Occupied(mut e) => {
                let vec = e.get_mut();

                if !vec.contains(v) {
                    vec.push(v.clone());
                    return true;
                }
            }
            Entry::Vacant(e) => {
                e.insert(vec![v.clone()]);
                return true;
            }
        };

        false
    }

    fn probe(&mut self, k: &Key, v: &ValProbe) {
        if let Some(entry) = self.table.get(k) {
            // TODO: We currently don't free/shrink the self.current_matches vecdeque to save time.
            // This mean it will grow to eventually become the largest number of matches in a single probe call.
            // Maybe we should clear this memory at the beginning of every tick/periodically?
            self.current_matches.extend(
                entry
                    .iter()
                    .map(|valbuild| (k.clone(), v.clone(), valbuild.clone())),
            );
        }
    }

    fn pop_match(&mut self) -> Option<(Key, ValProbe, ValBuild)> {
        self.current_matches.pop_front()
    }
}
