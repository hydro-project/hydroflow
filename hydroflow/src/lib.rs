#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![allow(type_alias_bounds)]
#![allow(clippy::let_and_return)]
#![allow(clippy::iter_with_drain)]
#![allow(clippy::explicit_auto_deref)]
// TODO(mingwei): Need rust-analyzer support
#![allow(clippy::uninlined_format_args)]

pub mod compiled;
pub mod lang;
pub mod props;
pub mod scheduled;
pub mod util;

pub use bytes;
pub use futures;
pub use pusherator;
pub use static_assertions;
pub use tokio;
pub use tokio_stream;
pub use tokio_util;
pub use tuple_list::tuple_list as tl;
pub use tuple_list::tuple_list_type as tt;

mod declarative_macro;
pub use declarative_macro::*;
pub use hydroflow_macro::*;

#[cfg(doctest)]
mod booktest {
    macro_rules! booktest {
        ($i:ident) => {
            #[doc = include_str!(concat!("../../book/", stringify!($i), ".md"))]
            mod $i {}
        };
    }

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
