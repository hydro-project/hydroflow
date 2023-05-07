use std::fmt::Debug;

use crate::{Merge, NaiveOrd};

#[allow(clippy::eq_op)]
#[allow(clippy::double_comparisons)]
pub fn assert_partial_ord_identities<T: PartialOrd + NaiveOrd>(test_vec: &[T]) {
    use std::cmp::Ordering::*;

    for a in test_vec {
        for b in test_vec {
            // `NaiveOrd` is a better source of truth, as it is based on the `Merge` impl. But it
            // is inefficient. It also could be wrong if `Merge` doesn't properly return true/false
            // iff the merge changed things.
            assert_eq!(a.naive_cmp(b), a.partial_cmp(b));
            assert_eq!(b.naive_cmp(a), b.partial_cmp(a));

            for c in test_vec {
                // Partial Eq:
                // a != b if and only if !(a == b).
                assert_eq!(a != b, !(a == b));

                // Eq:
                // reflexive: a == a;
                assert!(a == a);
                // symmetric: a == b implies b == a; and
                assert_eq!(a == b, b == a);
                // transitive: a == b and b == c implies a == c.
                if a == b && b == c {
                    assert_eq!(a == b && b == c, a == c);
                }

                // PartialOrd
                // a == b if and only if partial_cmp(a, b) == Some(Equal).
                assert_eq!(a == b, a.partial_cmp(b) == Some(Equal));
                // a < b if and only if partial_cmp(a, b) == Some(Less)
                assert_eq!(a < b, a.partial_cmp(b) == Some(Less));
                // a > b if and only if partial_cmp(a, b) == Some(Greater)
                assert_eq!(a > b, a.partial_cmp(b) == Some(Greater));
                // a <= b if and only if a < b || a == b
                assert_eq!(a <= b, a < b || a == b);
                // a >= b if and only if a > b || a == b
                assert_eq!(a >= b, a > b || a == b);
                // a != b if and only if !(a == b).
                assert_eq!(a != b, !(a == b));
                // transitivity: a < b and b < c implies a < c. The same must hold for both == and >.
                if a < b && b < c {
                    assert!(a < c);
                }
                if a == b && b == c {
                    assert!(a == c);
                }
                if a > b && b > c {
                    assert!(a > c);
                }
                // duality: a < b if and only if b > a.
                assert_eq!(a < b, b > a);
            }
        }
    }
}

pub fn assert_lattice_identities<T: Merge<T> + Clone + Eq + Debug>(test_vec: &[T]) {
    for x in test_vec {
        for y in test_vec {
            for z in test_vec {
                // Associativity
                // x ∧ (y ∧ z) = (x ∧ y) ∧ z
                assert_eq!(
                    Merge::merge_owned(
                        x.to_owned(),
                        Merge::merge_owned(y.to_owned(), z.to_owned())
                    ),
                    Merge::merge_owned(
                        Merge::merge_owned(x.to_owned(), y.to_owned()),
                        z.to_owned()
                    )
                );

                // Commutativity
                // x ∧ y = y ∧ x
                assert_eq!(
                    Merge::merge_owned(x.to_owned(), y.to_owned()),
                    Merge::merge_owned(y.to_owned(), x.to_owned())
                );

                // Idempotency
                // x ∧ x = x
                assert_eq!(Merge::merge_owned(x.to_owned(), x.to_owned()), x.to_owned());
            }
        }
    }
}
