pub use tuple_list::{tuple_list, tuple_list_type};
pub use tuple_list::{tuple_list as tl, tuple_list_type as tt};

#[macro_export]
macro_rules! variadic {
    (
        $( #[$( $attrs:tt )*] )*
        $vis:vis variadic<$item:ident> $name:ident $( $clause:tt )*
    ) => {
        $( #[$( $attrs )*] )*
        $vis trait $name: $crate::Variadic {}
        impl $name for () {}
        impl<$item, __Rest: $name> $name for ($item, __Rest) $( $clause )*
    };
}

pub trait Variadic {}
impl Variadic for () {}
impl<X, T> Variadic for (X, T) where T: Variadic {}

pub trait Extend<U>: Variadic
where
    U: Variadic,
{
    type Extended: Variadic;
    fn extend(self, input: U) -> Self::Extended;
}

impl<X, T, U> Extend<U> for (X, T)
where
    T: Variadic + Extend<U>,
    U: Variadic,
{
    type Extended = (X, <T as Extend<U>>::Extended);
    fn extend(self, input: U) -> Self::Extended {
        let (x, t) = self;
        (x, t.extend(input))
    }
}
impl<U> Extend<U> for ()
where
    U: Variadic,
{
    type Extended = U;
    fn extend(self, input: U) -> Self::Extended {
        input
    }
}

pub trait SplitPrefix<U>: Variadic
where
    U: Variadic,
{
    type Suffix: Variadic;
    fn split(self) -> (U, Self::Suffix);
}
impl<X, T, U> SplitPrefix<(X, U)> for (X, T)
where
    U: Variadic,
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
    T: Variadic,
{
    type Suffix = T;
    fn split(self) -> ((), Self::Suffix) {
        ((), self)
    }
}

pub trait Split<U, V>: Variadic
where
    U: Variadic,
    V: Variadic,
{
    fn split(self) -> (U, V);
}
impl<X, T, U, V> Split<(X, U), V> for (X, T)
where
    T: Split<U, V>,
    U: Variadic,
    V: Variadic,
{
    fn split(self) -> ((X, U), V) {
        let (x, t) = self;
        let (u, v) = t.split();
        ((x, u), v)
    }
}
impl<T> Split<(), T> for T
where
    T: Variadic,
{
    fn split(self) -> ((), T) {
        ((), self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type MyList = tt!(u8, u16, u32, u64);
    type MyPrefix = tt!(u8, u16);

    type MySuffix = <MyList as SplitPrefix<MyPrefix>>::Suffix;

    #[allow(dead_code)]
    const _: MySuffix = tl!(0_u32, 0_u64);
}

#[cfg(test)]
mod test_2 {
    use super::*;

    variadic! {
        #[doc = "A variadic list of `Debug` items"]
        pub variadic<T> DebugList where T: std::fmt::Debug {}
    }

    #[test]
    fn test() {
        let x = &tl!(1, "hello", 5.6);
        let _: &dyn DebugList = x;
        println!("{:?}", x);
    }
}
