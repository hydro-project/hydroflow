#![allow(non_camel_case_types)]
//! Emulation of higher-kinded types for collection type parameterization.
//! TODO(mingwei): Will be removed and replaced with `cc_traits` bounds in the futures.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::collections::{Array, MaskedArray, Single};

/// A trait (supposedly) for any type which has one type parameter.
pub trait Tag1<T> {
    /// The output of binding `Self` to `T`.
    type Bind;
}

/// A trait (supposedly) for any type which has two type parameters.
pub trait Tag2<T, U> {
    /// The output of binding `Self` to `T`, `U`.
    type Bind;
}

/// A [`HashSet`] tag.
pub enum HASH_SET {}
impl<T> Tag1<T> for HASH_SET {
    type Bind = HashSet<T>;
}

/// A [`HashMap`] tag.
pub enum HASH_MAP {}
impl<T, U> Tag2<T, U> for HASH_MAP {
    type Bind = HashMap<T, U>;
}

/// A [`BTreeSet`] tag.
pub enum BTREE_SET {}
impl<T> Tag1<T> for BTREE_SET {
    type Bind = BTreeSet<T>;
}

/// A [`BTreeMap`] tag.
pub enum BTREE_MAP {}
impl<T, U> Tag2<T, U> for BTREE_MAP {
    type Bind = BTreeMap<T, U>;
}

/// A [`Vec`] tag.
pub enum VEC {}
impl<T> Tag1<T> for VEC {
    type Bind = Vec<T>;
}
impl<T, U> Tag2<T, U> for VEC {
    type Bind = Vec<(T, U)>;
}

/// A [`crate::collections::Single`] tag.
pub enum SINGLE {}
impl<T> Tag1<T> for SINGLE {
    type Bind = Single<T>;
}
impl<T, U> Tag2<T, U> for SINGLE {
    type Bind = Single<(T, U)>;
}

/// An [`Option`] tag.
pub enum OPTION {}
impl<T> Tag1<T> for OPTION {
    type Bind = Option<T>;
}
impl<T, U> Tag2<T, U> for OPTION {
    type Bind = Option<(T, U)>;
}

/// A [`crate::collections::Array`] tag.
pub struct ARRAY<const N: usize>([(); N]);
impl<T, const N: usize> Tag1<T> for ARRAY<N> {
    type Bind = Array<T, N>;
}
impl<T, U, const N: usize> Tag2<T, U> for ARRAY<N> {
    type Bind = Array<(T, U), N>;
}

/// A [`crate::collections::MaskedArray`] tag.
pub struct MASKED_ARRAY<const N: usize>([(); N]);
impl<T, const N: usize> Tag1<T> for MASKED_ARRAY<N> {
    type Bind = MaskedArray<T, N>;
}
impl<T, U, const N: usize> Tag2<T, U> for MASKED_ARRAY<N> {
    type Bind = MaskedArray<(T, U), N>;
}
