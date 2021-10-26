use std::collections::HashMap;

pub struct SymmetricHashJoin<K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Clone,
    V2: Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    lhs: I1,
    ltab: HashMap<K, Vec<V1>>,
    rhs: I2,
    rtab: HashMap<K, Vec<V2>>,
    // TODO(justin): this shouldn't clone the buffer.
    lbuffer: Option<(K, V1, Vec<V2>)>,
    rbuffer: Option<(K, V2, Vec<V1>)>,
}
impl<K, I1, V1, I2, V2> Iterator for SymmetricHashJoin<K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Clone,
    V2: Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    type Item = (K, V1, V2);

    fn next(&mut self) -> Option<Self::Item> {
        let mut left_exhausted = false;
        let mut right_exhausted = false;
        while !left_exhausted
            && !right_exhausted
            && self.lbuffer.is_none()
            && self.rbuffer.is_none()
        {
            if let Some((k, v1)) = self.lhs.next() {
                self.ltab
                    .entry(k.clone())
                    .or_insert_with(Vec::new)
                    .push(v1.clone());
                if let Some(vs) = self.rtab.get(&k) {
                    self.lbuffer = Some((k, v1, vs.clone()));
                }
            } else {
                left_exhausted = true;
                if let Some((k, v2)) = self.rhs.next() {
                    // TODO(justin): unnecessary clone (sometimes).
                    self.rtab
                        .entry(k.clone())
                        .or_insert_with(Vec::new)
                        .push(v2.clone());
                    if let Some(vs) = self.ltab.get(&k) {
                        self.rbuffer = Some((k, v2, vs.clone()));
                    }
                } else {
                    right_exhausted = true;
                }
            }
        }

        if let Some((k, v, vs)) = &mut self.lbuffer {
            // TODO(justin): unnecessary clone (sometimes).
            let result = (k.clone(), v.clone(), vs.pop().unwrap());
            if vs.is_empty() {
                self.lbuffer = None;
            }
            Some(result)
        } else if let Some((k, v, vs)) = &mut self.rbuffer {
            // TODO(justin): unnecessary clone (sometimes).
            let result = (k.clone(), vs.pop().unwrap(), v.clone());
            if vs.is_empty() {
                self.rbuffer = None;
            }
            Some(result)
        } else {
            None
        }
    }
}
impl<K, I1, V1, I2, V2> SymmetricHashJoin<K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Clone,
    V2: Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    pub fn new(lhs: I1, rhs: I2) -> Self {
        SymmetricHashJoin {
            lhs,
            ltab: HashMap::new(),
            rhs,
            rtab: HashMap::new(),
            lbuffer: None,
            rbuffer: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compiled::pull::SymmetricHashJoin;

    #[test]
    fn hash_join() {
        let lhs = (0..10).map(|x| (x, format!("left {}", x)));
        let rhs = (6..15).map(|x| (x / 2, format!("right {} / 2", x)));

        let join = SymmetricHashJoin::new(lhs, rhs);

        assert_eq!(
            join.collect::<Vec<_>>(),
            vec![
                (3, "left 3".into(), "right 6 / 2".into()),
                (3, "left 3".into(), "right 7 / 2".into()),
                (4, "left 4".into(), "right 8 / 2".into()),
                (4, "left 4".into(), "right 9 / 2".into()),
                (5, "left 5".into(), "right 10 / 2".into()),
                (5, "left 5".into(), "right 11 / 2".into()),
                (6, "left 6".into(), "right 12 / 2".into()),
                (6, "left 6".into(), "right 13 / 2".into()),
                (7, "left 7".into(), "right 14 / 2".into())
            ]
        );
    }
}
