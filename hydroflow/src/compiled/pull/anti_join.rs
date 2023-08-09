use rustc_hash::FxHashSet;
pub struct AntiJoin<'a, Key, V, Ipos>
where
    Key: Eq + std::hash::Hash + Clone,
    V: Eq + std::hash::Hash + Clone,
    Ipos: Iterator<Item = (Key, V)>,
{
    input_pos: Ipos,
    neg_state: &'a mut FxHashSet<Key>,
    pos_state: &'a mut FxHashSet<(Key, V)>,
    old_state: std::vec::IntoIter<(Key, V)>,
}

impl<'a, Key, V, Ipos> Iterator for AntiJoin<'a, Key, V, Ipos>
where
    Key: Eq + std::hash::Hash + Clone,
    V: Eq + std::hash::Hash + Clone,
    Ipos: Iterator<Item = (Key, V)>,
{
    type Item = (Key, V);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.old_state.by_ref() {
            if !self.neg_state.contains(&item.0) {
                return Some(item);
            }
        }

        for item in self.input_pos.by_ref() {
            if !self.neg_state.contains(&item.0) && !self.pos_state.contains(&item) {
                self.pos_state.insert(item.clone());
                return Some(item);
            }
        }

        None
    }
}

impl<'a, Key, V, Ipos> AntiJoin<'a, Key, V, Ipos>
where
    Key: Eq + std::hash::Hash + Clone,
    V: Eq + std::hash::Hash + Clone,
    Ipos: Iterator<Item = (Key, V)>,
{
    pub fn new(
        input_pos: Ipos,
        state: &'a mut (&'a mut FxHashSet<Key>, &'a mut FxHashSet<(Key, V)>),
        new_tick: bool,
    ) -> Self {
        Self::new_from_mut(input_pos, state.0, state.1, new_tick)
    }

    pub fn new_from_mut(
        input_pos: Ipos,
        state_neg: &'a mut FxHashSet<Key>,
        state_pos: &'a mut FxHashSet<(Key, V)>,
        new_tick: bool,
    ) -> Self {
        let old_state = if new_tick {
            state_pos
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>()
        } else {
            Default::default()
        };

        Self {
            input_pos,
            neg_state: state_neg,
            pos_state: state_pos,
            old_state: old_state.into_iter(),
        }
    }
}
