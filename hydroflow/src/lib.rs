#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![allow(type_alias_bounds)]
#![allow(clippy::let_and_return)]
#![allow(clippy::iter_with_drain)]

pub mod builder;
pub mod compiled;
pub mod lang;
pub mod scheduled;

pub use tokio;
pub use tuple_list::tuple_list as tl;
pub use tuple_list::tuple_list_type as tt;

#[cfg(doctest)]
mod booktest {
    macro_rules! booktest {
        ($i:ident) => {
            #[doc = include_str!(concat!("../../book/", stringify!($i), ".md"))]
            mod $i {}
        };
    }
    booktest!(example_1);
    booktest!(example_2);
    booktest!(example_3);
    booktest!(example_4);
    booktest!(example_5);
}
