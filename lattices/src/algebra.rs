use std::fmt::Debug;

use crate::test::cartesian_power;

/// Defines a monoid structure.
/// A monoid is a set of items along with an associative binary operation `f` and an identity element `zero`.
/// The `f` operation combines two items and the `zero` element acts as the identity for `f`.
pub fn monoid<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S, // zero is the identity element of f
) {
    semigroup(items, f);
    identity(items, f, zero);
}

/// Defines a semigroup structure.
/// A semigroup is a set of items along with an associative binary operation `f`.
pub fn semigroup<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
) {
    associativity(items, f);
}

/// Defines a semiring structure.
/// A semiring is a set of items along with two associative binary operations `f` and `g`,
/// and two identity elements `zero` and `one`.
/// f must be commutative and g must distribute over f.
/// the zero of f must also be absorbing with respect to g.
pub fn semiring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, // zero is the identity element of f
    one: S,  // one is the identity element of g
) {
    commutative_monoid(items, f, zero.clone());
    monoid(items, g, one.clone());

    absorbing_element(items, g, zero);

    distributive(items, f, g);
}

/// Defines a ring structure.
/// A ring is a semiring with an inverse operation on f.
pub fn ring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, // zero is the identity element of f
    one: S,  // one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    semiring(items, f, g, zero.clone(), one);
    inverse(items, f, zero, b);
}

/// Defines an integral domain structure.
/// An integral domain is a nonzero commutative ring with no nonzero zero divisors.
pub fn integral_domain<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, // zero is the identity element of f
    one: S,  // one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    commutative_ring(items, f, g, zero.clone(), one, b);
    no_nonzero_zero_divisors(items, f, zero);
}

/// Defines a no-nonzero-zero-divisors property.
/// An element a is a zero divisor if there exists a non-zero element b such that ab = 0 or ba = 0
pub fn no_nonzero_zero_divisors<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
) {
    for a in items {
        for b in items {
            if *a != zero && *b != zero {
                assert_ne!(f(a.clone(), b.clone()), zero);
                assert_ne!(f(b.clone(), a.clone()), zero);
            }
        }
    }
}



/// Defines a commutative ring structure.
/// A commutative ring is a ring where the multiplication operation g is commutative.
pub fn commutative_ring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, // zero is the identity element of f
    one: S,  // one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    semiring(items, f, g, zero.clone(), one);
    inverse(items, f, zero, b);
    commutativity(items, g);
}

/// Defines a field structure.
/// A field is a commutative ring where every element has a multiplicative inverse.
pub fn field<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, // zero is the identity element of f
    one: S,  // one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    ring(items, f, g, zero.clone(), one.clone(), b);
    nonzero_inverse(items, f, one, zero, b);
}

/// Defines a commutative monoid structure.
/// A commutative monoid is a monoid where the operation f is commutative.
pub fn commutative_monoid<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
) {
    monoid(items, f, zero);
    commutativity(items, f);
}

/// Defines a group structure.
/// A group is a set of items along with an associative binary operation `f`
/// an identity element `zero`
/// and every element has an inverse element with respect to `f`
pub fn group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,             // zero is the identity element of f
    b: &impl Fn(S) -> S, /* b is the function to compute the inverse element of an element with respect to f */
) {
    monoid(items, f, zero.clone());
    inverse(items, f, zero, b);
}

/// Defines an abelian group structure.
/// An abelian group is a group where the operation f is commutative.
pub fn abelian_group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
    b: &impl Fn(S) -> S, /* b is the function to compute the inverse element of an element with respect to f */
) {
    group(items, f, zero, b);
    commutativity(items, f);
}

// Algebraic Properties
/// Defines the distributive property
/// a(b+c) = ab + ac
/// and (b+c)a = ba + ca
pub fn distributive<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
) {
    left_distributes(items, f, g);
    right_distributes(items, f, g);
}

/// Defines the left distributive property
/// a(b+c) = ab + ac
pub fn left_distributes<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    g: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        assert_eq!(
            g(a.clone(), f(b.clone(), c.clone())),
            f(g(a.clone(), b.clone()), g(a.clone(), c.clone()))
        );
    }
}

/// Defines the right distributive property.
/// (b+c)a = ba + ca
pub fn right_distributes<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    g: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        assert_eq!(
            g(f(b.clone(), c.clone()), a.clone()),
            f(g(b.clone(), a.clone()), g(c.clone(), a.clone()))
        );
    }
}

/// Defines the absorbing_element property.
/// An element z is absorbing if az = z and za = z for all a.
pub fn absorbing_element<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    z: S, // absorbing element (anything multiplied by z is z e.g. 0 in integers)
) {
    for a in items {
        // az = z
        assert_eq!(f(a.clone(), z.clone()), z.clone());

        // za = z
        assert_eq!(f(z.clone(), a.clone()), z.clone());
    }
}

/// Defines the inverse property.
/// An element b is the inverse of a if ab = e and ba = e for some identity element e.
pub fn inverse<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,               // e is the identity element of f
    b: impl Fn(S) -> S, /* b is the function to compute the inverse element of an element with respect to f */
) {
    // ∃b: ab = e, ba = e
    for a in items {
        assert_eq!(f(a.clone(), b(a.clone())), e);
        assert_eq!(f(b(a.clone()), a.clone()), e);
    }
}

/// Defines the non_zero inverse property.
/// Every element except zero must have an inverse.
pub fn nonzero_inverse<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
    zero: S,
    b: impl Fn(S) -> S,
) {
    // ∃b: ab = e, ba = e
    for a in items {
        if *a != zero {
            assert_eq!(f(a.clone(), b(a.clone())), e);
            assert_eq!(f(b(a.clone()), a.clone()), e);
        }
    }
}

/// Defines the identity property.
/// An element e is the identity of f if ae = a and ea = a for all a.
pub fn identity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
) {
    // ea = a, ae = a
    for a in items {
        assert_eq!(f(e.clone(), a.clone()), a.clone());
        assert_eq!(f(a.clone(), e.clone()), a.clone());
    }
}

/// Defines the associativity property.
/// a(bc) = (ab)c
pub fn associativity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        assert_eq!(
            f(a.clone(), f(b.clone(), c.clone())), // f(a, f(b,c)) ie a + (b + c)
            f(f(a.clone(), b.clone()), c.clone())  // f(f(a,b),c) ie (a + b) + c
        );
    }
}

/// Defines the commutativity property.
/// xy = yx
pub fn commutativity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    for [x, y] in cartesian_power(items) {
        assert_eq!(f(x.clone(), y.clone()), f(y.clone(), x.clone())); // a + b = b + a
    }
}

/// Defines the idempotency property.
/// xx = x
pub fn idempotency<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    for x in items {
        assert_eq!(f(x.clone(), x.clone()), x.clone());
    }
}

// Tests

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::algebra::*;

    static TEST_ITEMS: &[u32; 14] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

    #[test]
    fn test_associativity() {
        // Test that max() is associative and exponentiation isn't
        associativity(TEST_ITEMS, u32::max);
        assert!(std::panic::catch_unwind(|| {
            associativity(TEST_ITEMS, u32::wrapping_pow);
        })
        .is_err());
    }

    #[test]
    fn test_idempotency() {
        // Test that max() is idempotent and addition is non-idempotent
        idempotency(TEST_ITEMS, u32::max);

        assert!(std::panic::catch_unwind(|| {
            idempotency(TEST_ITEMS, u32::wrapping_add);
        })
        .is_err());
    }

    #[test]
    fn test_commutativity() {
        // Test that max() is commutative and division is non-commutative
        commutativity(TEST_ITEMS, u32::max);
        assert!(std::panic::catch_unwind(|| {
            commutativity(TEST_ITEMS, u32::wrapping_div);
        })
        .is_err());
    }

    #[test]
    fn test_identity() {
        // Test that 0 is the identity for addition and 5 is not
        identity(TEST_ITEMS, u32::wrapping_add, 0);

        assert!(std::panic::catch_unwind(|| {
            identity(TEST_ITEMS, u32::wrapping_add, 5);
        })
        .is_err());
    }

    #[test]
    fn test_inverse() {
        // Test that subtraction is the inverse of addition and that addition is not the inverse of addition
        inverse(TEST_ITEMS, u32::wrapping_add, 0, |x| 0u32.wrapping_sub(x));

        assert!(std::panic::catch_unwind(|| {
            inverse(TEST_ITEMS, u32::wrapping_add, 0, |x| 0u32.wrapping_add(x));
        })
        .is_err());
    }

    #[test]
    fn test_distributive() {
        // Test that addition and multiplication are distributive and that addition and max() are not
        distributive(TEST_ITEMS, &u32::wrapping_add, &u32::wrapping_mul);
        assert!(std::panic::catch_unwind(|| {
            distributive(TEST_ITEMS, &u32::wrapping_add, &u32::max);
        })
        .is_err());
    }

    #[test]
    fn test_absorbing() {
        // Test that 0 is absorbing for multiplication and 5 is not
        absorbing_element(TEST_ITEMS, u32::wrapping_mul, 0);
        assert!(std::panic::catch_unwind(|| {
            absorbing_element(TEST_ITEMS, u32::wrapping_mul, 5);
        })
        .is_err());
    }

    #[test]
    fn test_ring() {
        // Test that +, x, 0, 1, - are a ring and +, x, 0, 5 are not (5 isn't a multiplicative identity)
        ring(
            TEST_ITEMS,
            &u32::wrapping_add,
            &u32::wrapping_mul,
            0,
            1,
            &|x| 0u32.wrapping_sub(x),
        );
        assert!(std::panic::catch_unwind(|| {
            ring(
                TEST_ITEMS,
                &u32::wrapping_add,
                &u32::wrapping_mul,
                0,
                5,
                &|x| 0u32.wrapping_sub(x),
            );
        })
        .is_err());
    }

    #[test]
    fn test_semiring() {
        // Test +, x is a semiring
        semiring(TEST_ITEMS, &u32::wrapping_add, &u32::wrapping_mul, 0, 1);

        // Test boolean semiring with AND as + and OR as x
        semiring(&[false, true], &|x, y| x | y, &|x, y| x & y, false, true);

        // Test min plus semiring. + is min and x is plus. Also known as the "tropical semiring"
        semiring(
            &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::INFINITY],
            &f64::min,
            &|x, y| x + y,
            f64::INFINITY,
            0.0,
        );

        // Test max plus semiring. + is max and x is plus.
        semiring(
            &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::NEG_INFINITY],
            &f64::max,
            &|x, y| x + y,
            f64::NEG_INFINITY,
            0.0,
        );

        // Test sets of strings semiring with union as + and concatenation as x
        semiring(
            &[
                HashSet::from([]),
                HashSet::from(["".to_owned()]),
                HashSet::from(["a".to_owned()]),
                HashSet::from(["aa".to_owned(), "bb".to_owned()]),
                HashSet::from(["ab".to_owned(), "bb".to_owned(), "cc".to_owned()]),
                HashSet::from(["ba".to_owned()]),
                HashSet::from(["bb".to_owned()]),
            ],
            &|x, y| x.union(&y).cloned().collect(),
            &|x, y| {
                let mut new_set = HashSet::new();

                for a in x.iter() {
                    for b in y.iter() {
                        new_set.insert(format!("{a}{b}"));
                    }
                }

                new_set
            },
            HashSet::from([]),
            HashSet::from(["".to_owned()]),
        );
    }
}
