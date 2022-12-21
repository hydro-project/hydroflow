use super::clear::Clear;

/// A map-like interface which in reality only stored one value at a time. The
/// keys must be monotonically increasing (i.e. timestamps). For Hydroflow,
/// this allows state to be stored which resets each tick by using the
/// tick counter as the key. In the generic `Map` case it can be swapped out
/// for a true map to allow processing of multiple ticks of data at once.
#[derive(Clone, Copy, Debug)]
pub struct MonotonicMap<K, V>
where
    K: PartialOrd,
    V: Clear,
{
    key: Option<K>,
    val: V,
}

impl<K, V> Default for MonotonicMap<K, V>
where
    K: PartialOrd,
    V: Clear + Default,
{
    fn default() -> Self {
        Self {
            key: None,
            val: Default::default(),
        }
    }
}

impl<K, V> MonotonicMap<K, V>
where
    K: PartialOrd,
    V: Clear,
{
    /// Creates a new `MonotonicMap` initialized with the given value. The
    /// vaue will be `Clear`ed before it is accessed.
    pub fn new_init(val: V) -> Self {
        Self { key: None, val }
    }

    /// Inserts the value using the function if new `key` is strictly later than the current key.
    pub fn try_insert_with(&mut self, key: K, init: impl FnOnce() -> V) -> &mut V {
        if self
            .key
            .as_ref()
            .map(|old_key| old_key <= &key)
            .unwrap_or(true)
        {
            self.key = Some(key);
            self.val = (init)();
        }
        &mut self.val
    }

    /// Returns the value for the monotonically increasing key, or `None` if
    /// the key has already passed.
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        if self
            .key
            .as_ref()
            .map(|old_key| old_key <= &key)
            .unwrap_or(true)
        {
            self.key = Some(key);
            Some(&mut self.val)
        } else {
            None
        }
    }
}
