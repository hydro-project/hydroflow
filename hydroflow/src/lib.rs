#![feature(never_type)]
#![cfg_attr(feature = "variadic_generics", feature(generic_associated_types))]
#![allow(clippy::let_and_return)]

pub mod compiled;
pub mod lang;
pub mod scheduled;

pub use tuple_list::tuple_list as tl;
pub use tuple_list::tuple_list_type as tlt;
