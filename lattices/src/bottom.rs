//! Gives a default representation that compares as less than to a lattice.
//!
//! This can be used for giving a sensible default repersentation to types that don't necessarily have one.

use super::{Compare, ConvertFrom, Merge, Ordering};

#[derive(Clone, Debug, Default)]
#[repr(transparent)]
/// Bottom wrapper.
pub struct Bottom<Inner>(pub Option<Inner>);
impl<Inner> Bottom<Inner> {
    /// Create a new `Bottom` lattice instance from a value.
    pub fn new(val: Inner) -> Self {
        Self(Some(val))
    }

    /// Create a new `Bottom` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<Inner>) -> Self {
        Self::new(val.into())
    }
}

impl<Inner, Other> Merge<Bottom<Other>> for Bottom<Inner>
where
    Inner: Merge<Other> + ConvertFrom<Other>,
{
    fn merge(&mut self, other: Bottom<Other>) -> bool {
        match (&mut self.0, other.0) {
            (None, None) => false,
            (Some(_), None) => false,
            (this @ None, Some(other_inner)) => {
                *this = Some(ConvertFrom::from(other_inner));
                true
            }
            (Some(self_inner), Some(other_inner)) => self_inner.merge(other_inner),
        }
    }
}

impl<Inner> ConvertFrom<Bottom<Inner>> for Bottom<Inner> {
    fn from(other: Bottom<Inner>) -> Self {
        other
    }
}

impl<Inner, Other> Compare<Bottom<Other>> for Bottom<Inner>
where
    Inner: Compare<Other>,
{
    fn compare(&self, other: &Bottom<Other>) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (Some(this_inner), Some(other_inner)) => this_inner.compare(other_inner),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ord::Max;
    use crate::Ordering::*;

    type Bottom = super::Bottom<Max<usize>>;

    #[test]
    #[rustfmt::skip]
    fn merge() {
        assert!(Bottom::new(Max::new(0)).merge(Bottom::new(Max::new(1))));
        assert!(!Bottom::new(Max::new(1)).merge(Bottom::new(Max::new(0))));
        assert!(!Bottom::new(Max::new(0)).merge(Bottom::new(Max::new(0))));

        assert!(Bottom::default().merge(Bottom::new(Max::new(0))));
        assert!(!Bottom::new(Max::new(1)).merge(Bottom::default()));

        assert!(!Bottom::default().merge(Bottom::default()));
    }

    #[test]
    #[rustfmt::skip]
    fn compare() {
        assert_eq!(Bottom::new(Max::new(0)).compare(&Bottom::new(Max::new(1))), Some(Less));
        assert_eq!(Bottom::new(Max::new(1)).compare(&Bottom::new(Max::new(0))), Some(Greater));
        assert_eq!(Bottom::new(Max::new(0)).compare(&Bottom::new(Max::new(0))), Some(Equal));

        assert_eq!(Bottom::default().compare(&Bottom::new(Max::new(1))), Some(Less));
        assert_eq!(Bottom::new(Max::new(1)).compare(&Bottom::default()), Some(Greater));

        assert_eq!(Bottom::default().compare(&Bottom::default()), Some(Equal));
    }
}
