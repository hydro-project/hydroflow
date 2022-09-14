#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![allow(type_alias_bounds)]
#![allow(clippy::let_and_return)]
#![allow(clippy::iter_with_drain)]
#![allow(clippy::explicit_auto_deref)]

pub mod builder;
pub mod compiled;
pub mod lang;
pub mod props;
pub mod scheduled;

pub use hydroflow_macro::*;

pub use pusherator;
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

    booktest!(example_1_surface);
    booktest!(example_2_surface);
    booktest!(example_3_surface);
    booktest!(example_4_surface);
    booktest!(example_5_surface);

    booktest!(surface_syntax);
    booktest!(surface_embedding);
    booktest!(surface_flows);
    booktest!(surface_data);
    booktest!(surface_ops);
}
