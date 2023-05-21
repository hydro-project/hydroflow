#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![allow(type_alias_bounds)]
#![allow(clippy::let_and_return)]
#![allow(clippy::iter_with_drain)]
#![allow(clippy::explicit_auto_deref)]

pub mod compiled;
pub mod lang;
pub mod props;
pub mod scheduled;
pub mod util;

pub use variadics::{self, var_args, var_expr, var_type};
pub use {
    bincode, bytes, futures, lattices, pusherator, rustc_hash, serde, serde_json, tokio,
    tokio_stream, tokio_util,
};

mod declarative_macro;
pub use declarative_macro::*;
#[cfg(feature = "hydroflow_datalog")]
pub use hydroflow_datalog::*;
#[cfg(feature = "hydroflow_macro")]
pub use hydroflow_macro::*;

#[cfg(doctest)]
mod booktest {
    macro_rules! booktest {
        ($i:ident, $( $t:tt )*) => {
            #[doc = include_str!(concat!("../../docs/docs/", $( stringify!($t), )* stringify!($i), ".md"))]
            mod $i {}
        };
    }

    booktest!(example_1_simplest, quickstart/);
    booktest!(example_2_simple, quickstart/);
    booktest!(example_3_stream, quickstart/);
    booktest!(example_4_neighbors, quickstart/);
    booktest!(example_5_reachability, quickstart/);
    booktest!(example_6_unreachability, quickstart/);
    booktest!(example_7_echo_server, quickstart/);
    booktest!(example_8_chat_server, quickstart/);

    booktest!(index, syntax/);
    booktest!(surface_embedding, syntax/);
    booktest!(surface_flows, syntax/);
    booktest!(surface_data, syntax/);

    mod surface_ops {
        hydroflow_macro::surface_booktest_operators!();
    }
}
