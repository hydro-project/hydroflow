use std::collections::HashMap;

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
    ltab: &'a mut HashMap<K, Vec<V1>>,
    rtab: &'a mut HashMap<K, Vec<V2>>,
    // TODO(justin): this shouldn't clone the buffer.
    lbuffer: &'a mut Option<(K, V1, Vec<V2>)>,
    rbuffer: &'a mut Option<(K, V2, Vec<V1>)>,
}
impl<'a, K, I1, V1, I2, V2> Iterator for SymmetricHashJoin<'a, K, I1, V1, I2, V2>
where
    K: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (K, V1)>,
    I2: Iterator<Item = (K, V2)>,
{
    type Item = (K, V1, V2);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((k, v, vs)) = &mut self.lbuffer {
                // TODO(justin): unnecessary clone (sometimes).
                let result = (k.clone(), v.clone(), vs.pop().unwrap());
                if vs.is_empty() {
                    *self.lbuffer = None;
                }
                return Some(result);
            } else if let Some((k, v, vs)) = &mut self.rbuffer {
                // TODO(justin): unnecessary clone (sometimes).
                let result = (k.clone(), vs.pop().unwrap(), v.clone());
                if vs.is_empty() {
                    *self.rbuffer = None;
                }
                return Some(result);
            }

            if let Some((k, v1)) = self.lhs.next() {
                let vec = self.ltab.entry(k.clone()).or_insert_with(Vec::new);
                if !vec.contains(&v1) {
                    vec.push(v1.clone());
                    if let Some(vs) = self.rtab.get(&k) {
                        *self.lbuffer = Some((k, v1, vs.clone()));
                    }
                }
                continue;
            }

            if let Some((k, v2)) = self.rhs.next() {
                let vec = self.rtab.entry(k.clone()).or_insert_with(Vec::new);
                if !vec.contains(&v2) {
                    vec.push(v2.clone());
                    if let Some(vs) = self.ltab.get(&k) {
                        *self.rbuffer = Some((k, v2, vs.clone()));
                    }
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
    pub fn new(
        lhs: I1,
        rhs: I2,
        ltab: &'a mut HashMap<K, Vec<V1>>,
        rtab: &'a mut HashMap<K, Vec<V2>>,
        lbuffer: &'a mut Option<(K, V1, Vec<V2>)>,
        rbuffer: &'a mut Option<(K, V2, Vec<V1>)>,
    ) -> Self {
        Self {
            lhs,
            ltab,
            rhs,
            rtab,
            lbuffer,
            rbuffer,
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

        let (mut ltab, mut rtab, mut lbuf, mut rbuf) = Default::default();

        let join = SymmetricHashJoin::new(lhs, rhs, &mut ltab, &mut rtab, &mut lbuf, &mut rbuf);

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
