pub trait TypeList {}

impl TypeList for () {}
impl<X, T> TypeList for (X, T) where T: TypeList {}

pub trait Extend<U>: TypeList
where
    U: TypeList,
{
    type Extended: TypeList;
    fn extend(self, input: U) -> Self::Extended;
}

impl<X, T, U> Extend<U> for (X, T)
where
    T: TypeList + Extend<U>,
    U: TypeList,
{
    type Extended = (X, <T as Extend<U>>::Extended);
    fn extend(self, input: U) -> Self::Extended {
        let (x, t) = self;
        (x, t.extend(input))
    }
}
impl<U> Extend<U> for ()
where
    U: TypeList,
{
    type Extended = U;
    fn extend(self, input: U) -> Self::Extended {
        input
    }
}

pub trait SplitPrefix<U>: TypeList
where
    U: TypeList,
{
    type Suffix: TypeList;
    fn split(self) -> (U, Self::Suffix);
}
impl<X, T, U> SplitPrefix<(X, U)> for (X, T)
where
    U: TypeList,
    T: SplitPrefix<U>,
{
    type Suffix = <T as SplitPrefix<U>>::Suffix;
    fn split(self) -> ((X, U), Self::Suffix) {
        let (x, t) = self;
        let (t, u) = t.split();
        ((x, t), u)
    }
}
impl<T> SplitPrefix<()> for T
where
    T: TypeList,
{
    type Suffix = T;
    fn split(self) -> ((), Self::Suffix) {
        ((), self)
    }
}

pub trait Split<U, V>: TypeList
where
    U: TypeList,
    V: TypeList,
{
    fn split(self) -> (U, V);
}
impl<X, T, U, V> Split<(X, U), V> for (X, T)
where
    T: Split<U, V>,
    U: TypeList,
    V: TypeList,
{
    fn split(self) -> ((X, U), V) {
        let (x, t) = self;
        let (u, v) = t.split();
        ((x, u), v)
    }
}
impl<T> Split<(), T> for T
where
    T: TypeList,
{
    fn split(self) -> ((), T) {
        ((), self)
    }
}

#[cfg(test)]
mod test {
    use crate::{tl, tt};

    use super::*;

    type MyList = tt!(u8, u16, u32, u64);
    type MyPrefix = tt!(u8, u16);

    type MySuffix = <MyList as SplitPrefix<MyPrefix>>::Suffix;

    #[allow(dead_code)]
    const _: MySuffix = tl!(0_u32, 0_u64);
}
