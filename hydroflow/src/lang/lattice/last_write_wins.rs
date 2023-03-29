use std::fmt::Debug;
use std::{cmp::Ordering, marker::PhantomData};

use super::Convert;
use crate::lang::lattice::{Compare, Lattice, LatticeRepr, Merge};

pub struct LastWriteWins<M: Ord, T> {
    _phantom: PhantomData<(M, T)>,
}
impl<M: Ord, T> Lattice for LastWriteWins<M, T> {}

pub struct LastWriteWinsRepr<M: Ord, T> {
    _phantom: PhantomData<(M, T)>,
}
impl<M: Ord + Clone, T: Clone> LatticeRepr for LastWriteWinsRepr<M, T> {
    type Lattice = LastWriteWins<M, T>;
    type Repr = (M, T);
}

impl<M: Ord + Clone, T: Clone + PartialEq + Debug> Merge<LastWriteWinsRepr<M, T>>
    for LastWriteWinsRepr<M, T>
{
    fn merge(
        this: &mut <LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
        delta: <LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
    ) -> bool {
        let mut changed = false;

        let (marker_this, payload_this) = this;
        let (marker_delta, payload_delta) = delta;

        match marker_delta.cmp(marker_this) {
            Ordering::Less => {
                // do nothing.
            }
            Ordering::Equal => {
                // If markers are equal then payloads must be equal
                // Maybe an additional node-id tag could be added
                // to ensure a total order for when the marker is tied?
                debug_assert_eq!(*payload_this, payload_delta);
            }
            Ordering::Greater => {
                changed = true;
                *marker_this = marker_delta;
                *payload_this = payload_delta;
            }
        }

        changed
    }
}

impl<M: Ord + Clone, T: Clone + PartialEq + Debug> Compare<LastWriteWinsRepr<M, T>>
    for LastWriteWinsRepr<M, T>
{
    fn compare(
        this: &<LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
        delta: &<LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
    ) -> Option<Ordering> {
        let (marker_this, payload_this) = this;
        let (marker_delta, payload_delta) = delta;

        let ordering = marker_this.cmp(marker_delta);

        if ordering == Ordering::Equal {
            // If markers are equal then payloads must be equal
            debug_assert_eq!(*payload_this, *payload_delta);
        }

        Some(ordering)
    }
}

impl<M: Ord + Clone, T: Clone> Convert<LastWriteWinsRepr<M, T>> for LastWriteWinsRepr<M, T> {
    fn convert(
        this: <LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
    ) -> <LastWriteWinsRepr<M, T> as LatticeRepr>::Repr {
        this
    }
}

#[cfg(test)]
mod tests {
    use super::LastWriteWinsRepr;
    use crate::lang::lattice::{Compare, Convert, Merge};
    use std::cmp::Ordering;

    type Lww = LastWriteWinsRepr<usize, usize>;

    #[test]
    fn merge_lattice_moves_forward() {
        let mut x = (0, 0);
        assert!(<Lww as Merge<Lww>>::merge(&mut x, (1, 2)));
        assert_eq!(x, (1, 2));
    }

    #[test]
    fn merge_lattice_doesnt_move_backward() {
        let mut x = (1, 2);
        assert!(!<Lww as Merge<Lww>>::merge(&mut x, (0, 0)));
        assert_eq!(x, (1, 2));
    }

    #[test]
    #[should_panic]
    fn merge_equal_marker_different_payload_panics() {
        let mut x = (0, 0);
        assert!(!<Lww as Merge<Lww>>::merge(&mut x, (0, 1)));
        assert_eq!(x, (0, 0));
    }

    #[test]
    fn merge_equal_marker_same_payload_does_not_panic() {
        let mut x = (3, 3);
        assert!(!<Lww as Merge<Lww>>::merge(&mut x, (3, 3)));
        assert_eq!(x, (3, 3));
    }

    #[test]
    fn compare() {
        assert_eq!(
            <Lww as Compare<Lww>>::compare(&(0, 0), &(0, 0)),
            Some(Ordering::Equal)
        );

        assert_eq!(
            <Lww as Compare<Lww>>::compare(&(0, 0), &(1, 0)),
            Some(Ordering::Less)
        );

        assert_eq!(
            <Lww as Compare<Lww>>::compare(&(1, 0), &(0, 0)),
            Some(Ordering::Greater)
        );
    }

    #[test]
    #[should_panic]
    fn compare_equal_marker_different_payload_panics() {
        <Lww as Compare<Lww>>::compare(&(0, 0), &(0, 1));
    }

    #[test]
    fn convert() {
        assert_eq!(<Lww as Convert<Lww>>::convert((0, 0)), (0, 0));
    }
}
