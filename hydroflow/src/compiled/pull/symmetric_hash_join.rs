use std::collections::hash_map::Entry;

type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

#[derive(Debug)]
pub struct HalfJoinState<Key, ValBuild, ValProbe> {
    /// Table to probe, vec val contains all matches.
    table: HashMap<Key, Vec<ValBuild>>,
    /// Not-yet emitted matches. [`Option`] of the `Key`, other-side probe value, and index within
    /// the corresponding `table[key]` vec.
    current_matches: Option<(Key, ValProbe, usize, *const Vec<ValBuild>)>,
}
impl<Key, ValBuild, ValProbe> Default for HalfJoinState<Key, ValBuild, ValProbe> {
    fn default() -> Self {
        Self {
            table: HashMap::default(),
            current_matches: None,
        }
    }
}
impl<Key, ValBuild, ValProbe> HalfJoinState<Key, ValBuild, ValProbe>
where
    Key: Clone + Eq + std::hash::Hash,
    ValBuild: Clone + Eq,
    ValProbe: Clone,
{
    fn pop_buffer(&mut self) -> Option<(Key, ValProbe, ValBuild)> {
        let (k, v, idx, cached_entry) = self.current_matches.as_mut()?;

        let vec = unsafe { &**cached_entry };

        let result = (k.clone(), v.clone(), vec[*idx].clone());
        *idx += 1;
        if vec.len() <= *idx {
            self.current_matches = None;
        }
        Some(result)
    }

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

    fn probe(&mut self, k: Key, v: ValProbe) {
        if let Some(entry) = self.table.get(&k) {
            self.current_matches = Some((k, v, 0, entry as *const _));
        }
    }
}

pub type JoinState<Key, V1, V2> = (HalfJoinState<Key, V1, V2>, HalfJoinState<Key, V2, V1>);
pub type JoinStateMut<'a, Key, V1, V2> = (
    &'a mut HalfJoinState<Key, V1, V2>,
    &'a mut HalfJoinState<Key, V2, V1>,
);

pub struct SymmetricHashJoin<'a, Key, I1, V1, I2, V2>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
{
    lhs: I1,
    rhs: I2,
    state: JoinStateMut<'a, Key, V1, V2>,
}

impl<'a, Key, I1, V1, I2, V2> Iterator for SymmetricHashJoin<'a, Key, I1, V1, I2, V2>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
{
    type Item = (Key, (V1, V2));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((k, v2, v1)) = self.state.0.pop_buffer() {
                return Some((k, (v1, v2)));
            }
            if let Some((k, v1, v2)) = self.state.1.pop_buffer() {
                return Some((k, (v1, v2)));
            }

            if let Some((k, v1)) = self.lhs.next() {
                if self.state.0.build(k.clone(), &v1) {
                    self.state.1.probe(k, v1);
                }
                continue;
            }
            if let Some((k, v2)) = self.rhs.next() {
                if self.state.1.build(k.clone(), &v2) {
                    self.state.0.probe(k, v2);
                }
                continue;
            }

            return None;
        }
    }
}
impl<'a, Key, I1, V1, I2, V2> SymmetricHashJoin<'a, Key, I1, V1, I2, V2>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
{
    pub fn new(lhs: I1, rhs: I2, state: &'a mut JoinState<Key, V1, V2>) -> Self {
        Self {
            lhs,
            rhs,
            state: (&mut state.0, &mut state.1),
        }
    }

    pub fn new_from_mut(
        lhs: I1,
        rhs: I2,
        state_lhs: &'a mut HalfJoinState<Key, V1, V2>,
        state_rhs: &'a mut HalfJoinState<Key, V2, V1>,
    ) -> Self {
        Self {
            lhs,
            rhs,
            state: (state_lhs, state_rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{JoinState, SymmetricHashJoin};

    #[test]
    fn hash_join() {
        let lhs = (0..10).map(|x| (x, format!("left {}", x)));
        let rhs = (6..15).map(|x| (x / 2, format!("right {} / 2", x)));

        let mut state = JoinState::default();
        let join = SymmetricHashJoin::new(lhs, rhs, &mut state);

        assert_eq!(
            join.collect::<Vec<_>>(),
            vec![
                (3, ("left 3".into(), "right 6 / 2".into())),
                (3, ("left 3".into(), "right 7 / 2".into())),
                (4, ("left 4".into(), "right 8 / 2".into())),
                (4, ("left 4".into(), "right 9 / 2".into())),
                (5, ("left 5".into(), "right 10 / 2".into())),
                (5, ("left 5".into(), "right 11 / 2".into())),
                (6, ("left 6".into(), "right 12 / 2".into())),
                (6, ("left 6".into(), "right 13 / 2".into())),
                (7, ("left 7".into(), "right 14 / 2".into()))
            ]
        );
    }
}
