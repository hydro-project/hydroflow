#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
//! &nbsp;
//!
//! # Main Macros
//!
//! ## [`var_expr!`]
#![doc = include_str!("../var_expr.md")]
//! ## [`var_type!`]
#![doc = include_str!("../var_type.md")]
//! ## [`var_args!`]
#![doc = include_str!("../var_args.md")]

use sealed::sealed;

#[doc = include_str!("../var_expr.md")]
#[macro_export]
macro_rules! var_expr {
    () => ( () );

    (...$a:ident $(,)? ) => ( $a );
    (...$a:expr  $(,)? ) => ( $a );
    (...$a:ident, $( $b:tt )+) => ( $crate::VariadicExt::extend($a, $crate::var_expr!( $( $b )* )) );
    (...$a:expr,  $( $b:tt )+) => ( $crate::VariadicExt::extend($a, $crate::var_expr!( $( $b )* )) );

    ($a:ident $(,)? ) => ( ($a, ()) );
    ($a:expr  $(,)? ) => ( ($a, ()) );
    ($a:ident, $( $b:tt )+) => ( ($a, $crate::var_expr!( $( $b )* )) );
    ($a:expr,  $( $b:tt )+) => ( ($a, $crate::var_expr!( $( $b )* )) );
}

#[doc = include_str!("../var_type.md")]
#[macro_export]
macro_rules! var_type {
    () => ( () );

    (...$a:ty $(,)? ) => ( $a );
    (...$a:ty, $( $b:tt )+) => ( <$a as $crate::VariadicExt>::Extend::<$crate::var_type!( $( $b )* )> );

    ($a:ty $(,)? ) => ( ($a, ()) );
    ($a:ty, $( $b:tt )+) => ( ($a, $crate::var_type!( $( $b )* )) );
}

#[doc = include_str!("../var_args.md")]
#[macro_export]
macro_rules! var_args {
    () => ( () );

    (...$a:pat $(,)? ) => ( $a );
    (...$a:ty, $( $b:tt )+) => ( ::core::compile_error!("`var_args!` can only have the `...` spread syntax on the last field.") );

    ($a:pat $(,)? ) => ( ($a, ()) );
    ($a:pat, $( $b:tt )+) => ( ($a, $crate::var_args!( $( $b )* )) );
}

/// This macro generates a basic variadic trait where each element must fulfill the `where` clause.
///
/// ```rust
/// use variadics::{var_expr, variadic_trait};
///
/// variadic_trait! {
///     /// A variadic list of `Debug` items.
///     pub variadic<Item> DebugList where Item: std::fmt::Debug {}
/// }
///
/// let x = &var_expr!(1, "hello", 5.6);
/// let _: &dyn DebugList = x;
/// println!("{:?}", x);
/// ```
///
/// This uses a special syntax similar to traits, but with the `trait` keyword replaced with
/// `variadic<T>` where `T` is the generic parameter name for each item in the variadic list. `T`
/// can be changed to any valid generic identifier. The bounds on `T` must be put in the where
/// clause; they cannot be expressed directly-- `variadic<T: Clone>` is invalid.
///
/// For now this can only create traits which bounds the `Item`s and cannot have associated
/// methods. This means the body of the variadic trait must be empty. But in the future this
/// declarative macro may be converted into a more powerful procedural macro with associated
/// method support.
#[macro_export]
macro_rules! variadic_trait {
    (
        $( #[$( $attrs:tt )*] )*
        $vis:vis variadic<$item:ident> $name:ident $( $clause:tt )*
    ) => {
        $( #[$( $attrs )*] )*
        $vis trait $name: $crate::Variadic {}
        $( #[$( $attrs )*] )*
        impl $name for $crate::var_type!() {}
        $( #[$( $attrs )*] )*
        impl<$item, __Rest: $name> $name for $crate::var_type!($item, ...__Rest) $( $clause )*
    };
}

/// A variadic tuple list.
///
/// This is a sealed trait, implemented only for `(Item, Rest) where Rest: Variadic` and `()`.
#[sealed]
pub trait Variadic {}
#[sealed]
impl<Item, Rest> Variadic for (Item, Rest) where Rest: Variadic {}
#[sealed]
impl Variadic for () {}

/// Extension methods/types for [`Variadic`]s.
///
/// This is a sealed trait.
#[sealed]
pub trait VariadicExt: Variadic {
    /// The number of items in this variadic (its length).
    const LEN: usize;

    /// Creates a new (longer) variadic type by appending `Suffix` onto the end of this variadc.
    type Extend<Suffix>: VariadicExt
    where
        Suffix: VariadicExt;
    /// Extends this variadic value by appending `suffix` onto the end.
    fn extend<Suffix>(self, suffix: Suffix) -> Self::Extend<Suffix>
    where
        Suffix: VariadicExt;

    /// The reverse of this variadic type.
    type Reverse: VariadicExt;
    /// Reverses this variadic value.
    fn reverse(self) -> Self::Reverse;
}
#[sealed]
impl<Item, Rest> VariadicExt for (Item, Rest)
where
    Rest: VariadicExt,
{
    const LEN: usize = 1 + Rest::LEN;

    type Extend<Suffix> = (Item, Rest::Extend<Suffix>) where Suffix: VariadicExt;
    fn extend<Suffix>(self, suffix: Suffix) -> Self::Extend<Suffix>
    where
        Suffix: VariadicExt,
    {
        let (item, rest) = self;
        (item, rest.extend(suffix))
    }

    type Reverse = <Rest::Reverse as VariadicExt>::Extend<(Item, ())>;
    fn reverse(self) -> Self::Reverse {
        let (item, rest) = self;
        rest.reverse().extend((item, ()))
    }
}
#[sealed]
impl VariadicExt for () {
    const LEN: usize = 0;

    type Extend<Suffix> = Suffix where Suffix: VariadicExt;
    fn extend<Suffix>(self, suffix: Suffix) -> Self::Extend<Suffix>
    where
        Suffix: VariadicExt,
    {
        suffix
    }

    type Reverse = ();
    fn reverse(self) -> Self::Reverse {}
}

/// A variadic where all elements are the same type, `T`.
///
/// This is a sealed trait.
#[sealed]
pub trait HomogenousVariadic<T>: Variadic {
    /// Returns a reference to an element.
    fn get(&self, i: usize) -> Option<&T>;
    /// Returns an exclusive reference to an element.
    fn get_mut(&mut self, i: usize) -> Option<&mut T>;

    /// Iterator type returned by `into_iter`.
    type IntoIter: Iterator<Item = T>;
    /// Turns this `HomogenousVariadic<T>` into an iterator of items `T`.
    fn into_iter(self) -> Self::IntoIter;
}
#[sealed]
impl<T> HomogenousVariadic<T> for () {
    fn get(&self, _i: usize) -> Option<&T> {
        None
    }
    fn get_mut(&mut self, _i: usize) -> Option<&mut T> {
        None
    }

    type IntoIter = std::iter::Empty<T>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}
#[sealed]
impl<T, Rest> HomogenousVariadic<T> for (T, Rest)
where
    Rest: HomogenousVariadic<T>,
{
    fn get(&self, i: usize) -> Option<&T> {
        let (item, rest) = self;
        if i == 0 {
            Some(item)
        } else {
            rest.get(i)
        }
    }
    fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        let (item, rest) = self;
        if i == 0 {
            Some(item)
        } else {
            rest.get_mut(i)
        }
    }

    type IntoIter = std::iter::Chain<std::iter::Once<T>, Rest::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        let (item, rest) = self;
        std::iter::once(item).chain(rest.into_iter())
    }
}

/// Helper trait for splitting a variadic into two parts. `Prefix` is the first part, everything
/// after is the `Suffix` or second part.
///
/// This is a sealed trait.
#[sealed]
pub trait Split<Prefix>: Variadic
where
    Prefix: Variadic,
{
    /// The second part when splitting this variadic by `Prefix`.
    type Suffix: Variadic;
    /// Splits this variadic into two parts, first the `Prefix`, and second the `Suffix`.
    fn split(self) -> (Prefix, Self::Suffix);
}
#[sealed]
impl<Item, Rest, PrefixRest> Split<(Item, PrefixRest)> for (Item, Rest)
where
    PrefixRest: Variadic,
    Rest: Split<PrefixRest>,
{
    type Suffix = <Rest as Split<PrefixRest>>::Suffix;
    fn split(self) -> ((Item, PrefixRest), Self::Suffix) {
        let (item, rest) = self;
        let (prefix_rest, suffix) = rest.split();
        ((item, prefix_rest), suffix)
    }
}
#[sealed]
impl<Rest> Split<()> for Rest
where
    Rest: Variadic,
{
    type Suffix = Rest;
    fn split(self) -> ((), Self::Suffix) {
        ((), self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type MyList = var_type!(u8, u16, u32, u64);
    type MyPrefix = var_type!(u8, u16);

    type MySuffix = <MyList as Split<MyPrefix>>::Suffix;

    #[allow(dead_code)]
    const _: MySuffix = var_expr!(0_u32, 0_u64);

    #[test]
    #[allow(clippy::let_unit_value)]
    fn test_basic_expr() {
        let _ = var_expr!();
        let _ = var_expr!(1);
        let _ = var_expr!(1, "b",);
        let _ = var_expr!("a",);
        let _ = var_expr!(false, true, 1 + 2);
    }

    variadic_trait! {
        /// Variaidic list of futures.
        pub variadic<F> FuturesList where F: std::future::Future {}
    }

    type _ListA = var_type!(u32, u8, i32);
    type _ListB = var_type!(..._ListA, bool, Option<()>);
    type _ListC = var_type!(..._ListA, bool, Option::<()>);
}
