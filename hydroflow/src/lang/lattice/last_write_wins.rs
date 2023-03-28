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

impl<M: Ord + Clone, T: Clone> Merge<LastWriteWinsRepr<M, T>> for LastWriteWinsRepr<M, T> {
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
                // But don't want to require T: PartialEq + Debug
                // assert_eq!(*payload_this, payload_delta)
                // Maybe an additional node-id tag could be added
                // to ensure a total order for when the marker is tied?
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

impl<M: Ord + Clone, T: Clone> Compare<LastWriteWinsRepr<M, T>> for LastWriteWinsRepr<M, T> {
    fn compare(
        this: &<LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
        delta: &<LastWriteWinsRepr<M, T> as LatticeRepr>::Repr,
    ) -> Option<Ordering> {
        let (marker_this, _) = this;
        let (marker_delta, _) = delta;

        // Could also do an extra assert here for when markers are equal that payload is also equal.
        Some(marker_this.cmp(marker_delta))
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
    use crate::lang::lattice::Merge;

    type Lww = LastWriteWinsRepr<usize, usize>;

    #[test]
    fn lattice_moves_forward() {
        let mut x = (0, 0);
        assert!(<Lww as Merge<Lww>>::merge(&mut x, (1, 2)));
        assert_eq!(x, (1, 2));
    }

    #[test]
    fn lattice_doesnt_move_backward() {
        let mut x = (1, 2);
        assert!(!<Lww as Merge<Lww>>::merge(&mut x, (0, 0)));
        assert_eq!(x, (1, 2));
    }

    #[test]
    fn equal_marker_does_nothing() {
        let mut x = (0, 0);
        assert!(!<Lww as Merge<Lww>>::merge(&mut x, (0, 1)));
        assert_eq!(x, (0, 0));
    }
}
