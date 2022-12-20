use std::collections::HashMap;

#[derive(Debug)]
pub struct HalfJoinState<K, VBuild, VProbe> {
    table: HashMap<K, Vec<VBuild>>,
    buffer: Option<(K, VProbe, usize)>,
}
impl<K, VBuild, VProbe> Default for HalfJoinState<K, VBuild, VProbe> {
    fn default() -> Self {
        Self {
            table: HashMap::new(),
            buffer: None,
        }
    }
}
impl<K, VBuild, VProbe> HalfJoinState<K, VBuild, VProbe>
where
    K: Clone + Eq + std::hash::Hash,
    VBuild: Clone + Eq,
    VProbe: Clone,
{
    fn pop_buffer(&mut self) -> Option<(K, VProbe, VBuild)> {
        let (k, v, idx) = self.buffer.as_mut()?;
        let row = &self.table[k];
        let result = (k.clone(), v.clone(), row[*idx].clone());
        *idx += 1;
        if row.len() <= *idx {
            self.buffer = None;
        }
        Some(result)
    }

    fn build(&mut self, k: K, v: &VBuild) -> bool {
        let vec = self.table.entry(k).or_insert_with(Vec::new);
        if !vec.contains(v) {
            vec.push(v.clone());
            return true;
        }
        false
    }

    fn probe(&mut self, k: K, v: VProbe) {
        if self.table.contains_key(&k) {
            self.buffer = Some((k, v, 0));
        }
    }
}

pub type JoinState<K, V1, V2> = (HalfJoinState<K, V1, V2>, HalfJoinState<K, V2, V1>);

pub struct SymmetricHashJoin<'a, K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    lhs: I1,
    rhs: I2,
    state: &'a mut JoinState<K, V1, V2>,
}

impl<'a, K, I1, V1, I2, V2> Iterator for SymmetricHashJoin<'a, K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    type Item = (K, (V1, V2));

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
impl<'a, K, I1, V1, I2, V2> SymmetricHashJoin<'a, K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    pub fn new(lhs: I1, rhs: I2, state: &'a mut JoinState<K, V1, V2>) -> Self {
        Self { lhs, rhs, state }
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
