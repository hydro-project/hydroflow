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

use std::any::Any;

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

/// return the (top-level) length of en explicit variadic
/// (cannot pass in a nested macro call like var_expr!(...))
#[macro_export]
macro_rules! var_len {
    // Match the empty tuple or base case
    (()) => {
        0
    };
    // Match a single element tuple, terminating the recursion
    (($elem:expr,())) => {
        1
    };
    // Match deeper nested tuples
    (($head:expr, $tail:tt)) => {
        1 + var_len!($tail)
    };
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

    /// This as a variadic of references.
    type AsRefVar<'a>: Copy + UnrefVariadic<Unref = Self>
    where
        Self: 'a;
    /// Convert a reference to this variadic into a variadic of references.
    fn as_ref_var(&self) -> Self::AsRefVar<'_>;

    /// This as a variadic of exclusive (`mut`) references.
    type AsMutVar<'a>: Variadic
    where
        Self: 'a;
    /// Convert an exclusive (`mut`) reference to this variadic into a variadic of exclusive
    /// (`mut`) references.
    fn as_mut_var(&mut self) -> Self::AsMutVar<'_>;

    /// Iterator type returned by [`Self::iter_any_ref`].
    type IterAnyRef<'a>: Iterator<Item = &'a dyn Any>
    where
        Self: 'static;
    /// Iterate this variadic as `&dyn Any` references.
    fn iter_any_ref(&self) -> Self::IterAnyRef<'_>
    where
        Self: 'static;

    /// Iterator type returned by [`Self::iter_any_mut`].
    type IterAnyMut<'a>: Iterator<Item = &'a mut dyn Any>
    where
        Self: 'static;
    /// Iterate this variadic as `&mut dyn Any` exclusive references.
    fn iter_any_mut(&mut self) -> Self::IterAnyMut<'_>
    where
        Self: 'static;
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

    type AsRefVar<'a> = (&'a Item, Rest::AsRefVar<'a>)
    where
        Self: 'a;
    fn as_ref_var(&self) -> Self::AsRefVar<'_> {
        let (item, rest) = self;
        (item, rest.as_ref_var())
    }

    type AsMutVar<'a> = (&'a mut Item, Rest::AsMutVar<'a>)
    where
        Self: 'a;
    fn as_mut_var(&mut self) -> Self::AsMutVar<'_> {
        let (item, rest) = self;
        (item, rest.as_mut_var())
    }

    type IterAnyRef<'a> = std::iter::Chain<std::iter::Once<&'a dyn Any>, Rest::IterAnyRef<'a>>
    where
        Self: 'static;
    fn iter_any_ref(&self) -> Self::IterAnyRef<'_>
    where
        Self: 'static,
    {
        let var_args!(item, ...rest) = self;
        let item: &dyn Any = item;
        std::iter::once(item).chain(rest.iter_any_ref())
    }

    type IterAnyMut<'a> = std::iter::Chain<std::iter::Once<&'a mut dyn Any>, Rest::IterAnyMut<'a>>
    where
        Self: 'static;
    fn iter_any_mut(&mut self) -> Self::IterAnyMut<'_>
    where
        Self: 'static,
    {
        let var_args!(item, ...rest) = self;
        let item: &mut dyn Any = item;
        std::iter::once(item).chain(rest.iter_any_mut())
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

    type AsRefVar<'a> = ();
    fn as_ref_var(&self) -> Self::AsRefVar<'_> {}

    type AsMutVar<'a> = ();
    fn as_mut_var(&mut self) -> Self::AsMutVar<'_> {}

    type IterAnyRef<'a> = std::iter::Empty<&'a dyn Any>
    where
        Self: 'static;
    fn iter_any_ref(&self) -> Self::IterAnyRef<'_>
    where
        Self: 'static,
    {
        std::iter::empty()
    }

    type IterAnyMut<'a> = std::iter::Empty<&'a mut dyn Any>
    where
        Self: 'static;
    fn iter_any_mut(&mut self) -> Self::IterAnyMut<'_>
    where
        Self: 'static,
    {
        std::iter::empty()
    }
}

/// Convert from a variadic of references back into the original variadic. The inverse of
/// [`VariadicExt::as_ref_var`] or [`VariadicExt::as_mut_var`].
///
/// This is a sealed trait.
#[sealed]
pub trait UnrefVariadic: Variadic {
    /// The un-referenced variadic. Each item will have one layer of references removed.
    type Unref: VariadicExt;
}
#[sealed]
impl<Item, Rest> UnrefVariadic for (&Item, Rest)
where
    Rest: UnrefVariadic,
{
    type Unref = (Item, Rest::Unref);
}
#[sealed]
impl<Item, Rest> UnrefVariadic for (&mut Item, Rest)
where
    Rest: UnrefVariadic,
{
    type Unref = (Item, Rest::Unref);
}
#[sealed]
impl UnrefVariadic for () {
    type Unref = ();
}

#[sealed]
/// Clone an Unref
pub trait UnrefCloneVariadic: UnrefVariadic {
    /// Clone the unref
    fn clone_var(&self) -> Self::Unref;
}
#[sealed]
impl<Item, Rest> UnrefCloneVariadic for (&Item, Rest)
where
    Item: Clone,
    Rest: UnrefCloneVariadic,
{
    fn clone_var(&self) -> Self::Unref {
        let var_args!(item, ...rest) = self;
        var_expr!((*item).clone(), ...rest.clone_var())
    }
}
#[sealed]
impl<Item, Rest> UnrefCloneVariadic for (&mut Item, Rest)
where
    Item: Clone,
    Rest: UnrefCloneVariadic,
{
    fn clone_var(&self) -> Self::Unref {
        let var_args!(item, ...rest) = self;
        var_expr!((*item).clone(), ...rest.clone_var())
    }
}
#[sealed]
impl UnrefCloneVariadic for () {
    fn clone_var(&self) -> Self::Unref {}
}

/// `PartialEq` between a referenced variadic and a variadic of references, of the same types.
#[sealed]
pub trait AsRefVariadicPartialEq: VariadicExt {
    /// `PartialEq` between a referenced variadic and a variadic of references, of the same types.
    fn as_ref_var_eq(&self, other: Self::AsRefVar<'_>) -> bool;
}
#[sealed]
impl<Item, Rest> AsRefVariadicPartialEq for (Item, Rest)
where
    Item: PartialEq,
    Rest: AsRefVariadicPartialEq,
{
    fn as_ref_var_eq(&self, other: Self::AsRefVar<'_>) -> bool {
        let var_args!(item_self, ...rest_self) = self;
        let var_args!(item_other, ...rest_other) = other;
        item_self == item_other && rest_self.as_ref_var_eq(rest_other)
    }
}
#[sealed]
impl AsRefVariadicPartialEq for () {
    fn as_ref_var_eq(&self, _other: Self::AsRefVar<'_>) -> bool {
        true
    }
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
        #[allow(dead_code)]
        pub variadic<F> FuturesList where F: std::future::Future {}
    }

    type _ListA = var_type!(u32, u8, i32);
    type _ListB = var_type!(..._ListA, bool, Option<()>);
    type _ListC = var_type!(..._ListA, bool, Option::<()>);

    #[test]
    fn test_as_ref_var() {
        let my_owned = var_expr!("Hello".to_owned(), Box::new(5));
        let my_ref_a = my_owned.as_ref_var();
        let my_ref_b = my_owned.as_ref_var();
        assert_eq!(my_ref_a, my_ref_b);
    }

    #[test]
    fn test_as_mut_var() {
        let mut my_owned = var_expr!("Hello".to_owned(), Box::new(5));
        let var_args!(mut_str, mut_box) = my_owned.as_mut_var();
        *mut_str += " World";
        *mut_box.as_mut() += 1;

        assert_eq!(var_expr!("Hello World".to_owned(), Box::new(6)), my_owned);
    }

    #[test]
    fn test_iter_any() {
        let mut var = var_expr!(1_i32, false, "Hello".to_owned());

        let mut mut_iter = var.iter_any_mut();
        *mut_iter.next().unwrap().downcast_mut::<i32>().unwrap() += 1;
        *mut_iter.next().unwrap().downcast_mut::<bool>().unwrap() |= true;
        *mut_iter.next().unwrap().downcast_mut::<String>().unwrap() += " World";
        assert!(mut_iter.next().is_none());

        let mut ref_iter = var.iter_any_ref();
        assert_eq!(
            Some(&2),
            ref_iter
                .next()
                .map(<dyn Any>::downcast_ref)
                .map(Option::unwrap)
        );
        assert_eq!(
            Some(&true),
            ref_iter
                .next()
                .map(<dyn Any>::downcast_ref)
                .map(Option::unwrap)
        );
        assert_eq!(
            Some("Hello World"),
            ref_iter
                .next()
                .map(|any| &**any.downcast_ref::<String>().unwrap())
        );
        assert!(ref_iter.next().is_none());
    }
}
