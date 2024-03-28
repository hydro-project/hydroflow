use std::collections::HashSet;
use std::fmt::Debug;

use itertools::{EitherOrBoth, Itertools};

use crate::test::cartesian_power;

static test_items: &[u32; 14] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

#[test]
fn test_associativity() {
    //Test that max() is associative and exponentiation isn't
    associativity(test_items, u32::max);
    assert!( std::panic::catch_unwind(|| {
        associativity(test_items, u32::wrapping_pow);
    }).is_err());
}

#[test]
fn test_idempotency() {
    //Test that max() is idempotent and addition is non-idempotent
    idempotency(test_items, u32::max);

    assert!( std::panic::catch_unwind(|| {
        idempotency(test_items, u32::wrapping_add);
    }).is_err());
}

#[test]
fn test_commutativity() {
    //Test that max() is commutative and division is non-commutative
    commutativity(test_items, u32::max);
    assert!( std::panic::catch_unwind(|| {
        commutativity(test_items, u32::wrapping_div);
    }).is_err());
}

#[test]
fn test_identity() {
    //Test that 0 is the identity for addition and 5 is not
    identity(test_items, u32::wrapping_add, 0);

    assert!( std::panic::catch_unwind(|| {
        identity(test_items, u32::wrapping_add, 5);
    }).is_err());
}


#[test]
fn test_inverse() {
    //Test that subtraction is the inverse of addition and that addition is not the inverse of addition
    inverse(test_items, u32::wrapping_add, 0, |x| 0u32.wrapping_sub(x));

    assert!( std::panic::catch_unwind(|| {
        inverse(test_items, u32::wrapping_add, 0, |x| 0u32.wrapping_add(x));
    }).is_err());
}

#[test]
fn test_distributive() {
    //Test that addition and multiplication are distributive and that addition and max() are not
    distributive(test_items, &u32::wrapping_add, &u32::wrapping_mul);
    assert!( std::panic::catch_unwind(|| {
        distributive(test_items, &u32::wrapping_add, &u32::max);
    }).is_err());
}

#[test]
fn test_absorbing() {
    // Test that 0 is absorbing for multiplication and 5 is not
    absorbing_element(test_items, &u32::wrapping_mul, 0);
    assert!( std::panic::catch_unwind(|| {
        absorbing_element(test_items, u32::wrapping_mul, 5);
    }).is_err());
}

#[test]
fn test_ring() {
    //Test that +, x, 0, 1, - are a ring and +, x, 0, 5 are not (5 isn't a multiplicative identity)
    ring(test_items, &u32::wrapping_add, &u32::wrapping_mul, 0, 1, &|x| {
        0u32.wrapping_sub(x)
    });
    assert!( std::panic::catch_unwind(|| {
        ring(test_items, &u32::wrapping_add, &u32::wrapping_mul, 0, 5, &|x| {
            0u32.wrapping_sub(x)
        });
    }).is_err());
}

#[test]
fn test_semiring(){
    //Test +, x is a semiring
    semiring(test_items, &u32::wrapping_add, &u32::wrapping_mul, 0, 1);

    //Test boolean semiring with AND as + and OR as x
    semiring(&[false, true], &|x, y| x | y, &|x, y| x & y, false, true);

    //Test min plus semiring. + is min and x is plus. Also known as the "tropical semiring"
    semiring(
        &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::INFINITY],
        &|x, y| f64::min(x, y),
        &|x, y| x + y,
        f64::INFINITY,
        0.0,
    );

    //Test max plus semiring. + is max and x is plus.
    semiring(
        &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::NEG_INFINITY],
        &|x, y| f64::max(x, y),
        &|x, y| x + y,
        f64::NEG_INFINITY,
        0.0,
    );

    //Test sets of strings semiring with union as + and concatenation as x
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

// Algebraic Structures
fn monoid<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S, //zero is the identity element of f
) {
    semigroup(items, f);
    identity(items, f, zero);
}

fn semigroup<S: Debug + PartialEq + Clone, const N: usize>(items: &[S; N], f: &impl Fn(S, S) -> S) {
    associativity(items, f);
}

fn semiring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, //zero is the identity element of f
    one: S, //one is the identity element of g
) {
    commutative_monoid(items, f, zero.clone());
    monoid(items, g, one.clone());

    absorbing_element(items, g, zero);

    distributive(items, f, g);
}

fn ring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, //zero is the identity element of f
    one: S, //one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    semiring(items, f, g, zero.clone(), one);
    inverse(items, f, zero, b);
}

fn commutative_ring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, //zero is the identity element of f
    one: S, //one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    semiring(items, f, g, zero.clone(), one);
    inverse(items, f, zero, b);
    commutativity(items, g);
}

fn field<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S, //zero is the identity element of f
    one: S, //one is the identity element of g
    b: &impl Fn(S) -> S,
) {
    ring(items, f, g, zero.clone(), one.clone(), b);
    nonzero_inverse(items, f, one, zero, b);
}

fn commutative_monoid<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
) {
    monoid(items, f, zero);
    commutativity(items, f);
}

fn abelian_group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
    b: &impl Fn(S) -> S, //b is the inverse function of f
) {
    group(items, f, zero, b);
    commutativity(items, f);
}

fn group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S, //zero is the identity element of f
    b: &impl Fn(S) -> S, //b is the inverse function of f
) {
    monoid(items, f, zero.clone());
    inverse(items, f, zero, b);
}


// Algebraic Properties
fn distributive<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
) {
    left_distributes(items, f, g);
    right_distributes(items, f, g);
}

fn left_distributes<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    g: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        // a(b+c) = ab + ac
        assert_eq!(
            g(a.clone(), f(b.clone(), c.clone())),
            f(g(a.clone(), b.clone()), g(a.clone(), c.clone()))
        );
    }
}

fn right_distributes<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    g: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        // (b+c)a = ba + ca
        assert_eq!(
            g(f(b.clone(), c.clone()), a.clone()),
            f(g(b.clone(), a.clone()), g(c.clone(), a.clone()))
        );
    }
}

fn absorbing_element<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    z: S, //absorbing element (anything multiplied by z is z e.g. 0 in integers)
) {
    for [a] in cartesian_power(items) {
        // az = z
        assert_eq!(f(a.clone(), z.clone()), z.clone());

        // za = z
        assert_eq!(f(z.clone(), a.clone()), z.clone());
    }
}

fn inverse<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S, //e is the identity element of f
    b: impl Fn(S) -> S, //b is the inverse function of f
) {
    // ∃b: ab = e, ba = e
    for [a] in cartesian_power(items) {
        assert_eq!(f(a.clone(), b(a.clone())), e);
        assert_eq!(f(b(a.clone()), a.clone()), e);
    }
}

fn nonzero_inverse<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
    zero: S,
    b: impl Fn(S) -> S,
) {
    // ∃b: ab = e, ba = e
    for [a] in cartesian_power(items) {
        if *a != zero {
            assert_eq!(f(a.clone(), b(a.clone())), e);
            assert_eq!(f(b(a.clone()), a.clone()), e);
        }
    }
}

fn identity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
) {
    // ea = a, ae = a
    for [a] in cartesian_power(items) {
        assert_eq!(f(e.clone(), a.clone()), a.clone());
        assert_eq!(f(a.clone(), e.clone()), a.clone());
    }
}

fn associativity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    // a(bc) = (ab)c
    for [a, b, c] in cartesian_power(items) {
        assert_eq!(
            f(a.clone(), f(b.clone(), c.clone())), //f(a, f(b,c)) ie a + (b + c)
            f(f(a.clone(), b.clone()), c.clone()) // f(f(a,b),c) ie (a + b) + c
        );
    }
}

fn commutativity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    // xy = yx
    for [x, y] in cartesian_power(items) {
        assert_eq!(f(x.clone(), y.clone()), f(y.clone(), x.clone())); // a + b = b + a
    }
}

fn idempotency<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    // xx = x
    for [x] in cartesian_power(items) {
        assert_eq!(f(x.clone(), x.clone()), x.clone());
    }
}

