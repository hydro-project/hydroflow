#![allow(non_camel_case_types)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::collections::{Single, Array, MaskedArray};

pub trait Tag1<T>: 'static {
    type Bind;
}

pub trait Tag2<T, U>: 'static {
    type Bind;
}


pub enum HASH_SET {}
impl<T> Tag1<T> for HASH_SET {
    type Bind = HashSet<T>;
}

pub enum HASH_MAP {}
impl<T, U> Tag2<T, U> for HASH_MAP {
    type Bind = HashMap<T, U>;
}


pub enum BTREE_SET {}
impl<T> Tag1<T> for BTREE_SET {
    type Bind = BTreeSet<T>;
}

pub enum BTREE_MAP {}
impl<T, U> Tag2<T, U> for BTREE_MAP {
    type Bind = BTreeMap<T, U>;
}


pub enum VEC {}
impl<T> Tag1<T> for VEC {
    type Bind = Vec<T>;
}
impl<T, U> Tag2<T, U> for VEC {
    type Bind = Vec<(T, U)>;
}


pub enum SINGLE {}
impl<T> Tag1<T> for SINGLE {
    type Bind = Single<T>;
}
impl<T, U> Tag2<T, U> for SINGLE {
    type Bind = Single<(T, U)>;
}


pub enum OPTION {}
impl<T> Tag1<T> for OPTION {
    type Bind = Option<T>;
}
impl<T, U> Tag2<T, U> for OPTION {
    type Bind = Option<(T, U)>;
}


pub struct ARRAY<const N: usize>([(); N]);
impl<T, const N: usize> Tag1<T> for ARRAY<N> {
    type Bind = Array<T, N>;
}
impl<T, U, const N: usize> Tag2<T, U> for ARRAY<N> {
    type Bind = Array<(T, U), N>;
}


pub struct MASKED_ARRAY<const N: usize>([(); N]);
impl<T, const N: usize> Tag1<T> for MASKED_ARRAY<N> {
    type Bind = MaskedArray<T, N>;
}
impl<T, U, const N: usize> Tag2<T, U> for MASKED_ARRAY<N> {
    type Bind = MaskedArray<(T, U), N>;
}
