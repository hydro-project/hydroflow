#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![allow(type_alias_bounds)]
#![allow(clippy::let_and_return)]
#![allow(clippy::iter_with_drain)]
#![allow(clippy::explicit_auto_deref)]

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
pub use type_list::{self, tl, tt};

mod declarative_macro;
pub use declarative_macro::*;
pub use hydroflow_macro::*;

#[cfg(doctest)]
mod booktest {
    macro_rules! booktest {
        ($i:ident $( $t:tt )*) => {
            #[doc = include_str!(concat!("../../book/", stringify!($i), $( stringify!($t), )* ".md"))]
            mod $i {}
        };
    }

    booktest!(example_1_surface);
    booktest!(example_2_surface);
    booktest!(example_3_surface);
    booktest!(example_4_surface);
    booktest!(example_5_surface);
    booktest!(example_6_surface);

    booktest!(surface_syntax);
    booktest!(surface_embedding);
    booktest!(surface_flows);
    booktest!(surface_data);

    mod surface_ops {
        hydroflow_macro::surface_booktest_operators!();
    }
}
