use super::HalfJoinState;

pub struct SymmetricHashJoin<'a, Key, I1, V1, I2, V2, LhsState, RhsState>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
    LhsState: HalfJoinState<Key, V1, V2>,
    RhsState: HalfJoinState<Key, V2, V1>,
{
    lhs: I1,
    rhs: I2,
    lhs_state: &'a mut LhsState,
    rhs_state: &'a mut RhsState,
}

impl<'a, Key, I1, V1, I2, V2, LhsState, RhsState> Iterator
    for SymmetricHashJoin<'a, Key, I1, V1, I2, V2, LhsState, RhsState>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
    LhsState: HalfJoinState<Key, V1, V2>,
    RhsState: HalfJoinState<Key, V2, V1>,
{
    type Item = (Key, (V1, V2));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((k, v2, v1)) = self.lhs_state.pop_match() {
                return Some((k, (v1, v2)));
            }
            if let Some((k, v1, v2)) = self.rhs_state.pop_match() {
                return Some((k, (v1, v2)));
            }

            if let Some((k, v1)) = self.lhs.next() {
                if self.lhs_state.build(k.clone(), &v1) {
                    self.rhs_state.probe(&k, &v1);
                }
                continue;
            }
            if let Some((k, v2)) = self.rhs.next() {
                if self.rhs_state.build(k.clone(), &v2) {
                    self.lhs_state.probe(&k, &v2);
                }
                continue;
            }

            return None;
        }
    }
}
impl<'a, Key, I1, V1, I2, V2, LhsState, RhsState>
    SymmetricHashJoin<'a, Key, I1, V1, I2, V2, LhsState, RhsState>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
    LhsState: HalfJoinState<Key, V1, V2>,
    RhsState: HalfJoinState<Key, V2, V1>,
{
    pub fn new(lhs: I1, rhs: I2, state: &'a mut (LhsState, RhsState)) -> Self {
        Self {
            lhs,
            rhs,
            lhs_state: &mut state.0,
            rhs_state: &mut state.1,
        }
    }

    pub fn new_from_mut(
        lhs: I1,
        rhs: I2,
        state_lhs: &'a mut LhsState,
        state_rhs: &'a mut RhsState,
    ) -> Self {
        Self {
            lhs,
            rhs,
            lhs_state: state_lhs,
            rhs_state: state_rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SymmetricHashJoin;
    use crate::compiled::pull::JoinState;

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

    #[test]
    fn hash_join_subsequent_ticks_dont_produce_if_nothing_is_changed() {
        let (lhs_tx, lhs_rx) = std::sync::mpsc::channel::<(usize, usize)>();
        let (rhs_tx, rhs_rx) = std::sync::mpsc::channel::<(usize, usize)>();

        let mut state = JoinState::default();
        let mut join = SymmetricHashJoin::new(lhs_rx.try_iter(), rhs_rx.try_iter(), &mut state);

        lhs_tx.send((7, 3)).unwrap();
        rhs_tx.send((7, 3)).unwrap();

        assert_eq!(join.next(), Some((7, (3, 3))));
        assert_eq!(join.next(), None);

        lhs_tx.send((7, 3)).unwrap();
        rhs_tx.send((7, 3)).unwrap();

        assert_eq!(join.next(), None);
    }
}
