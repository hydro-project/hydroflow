#![warn(missing_docs)]

//! Hydroflow is a low-level dataflow-based runtime system for the [Hydro Project](https://hydro.run/).
//!
//! The primary item in this crate is the [`Hydroflow`](crate::scheduled::graph::Dfir) struct,
//! representing a Hydroflow dataflow graph. Although this graph can be manually constructed, the
//! easiest way to instantiate a `Hydroflow` instance is with the [`dfir_syntax!`] macro using
//! Hydroflow's custom "surface syntax."
//!
//! ```rust
//! let mut hf = dfir_rs::dfir_syntax! {
//!     source_iter(["hello", "world"]) -> for_each(|s| println!("{}", s));
//! };
//! hf.run_available();
//! ```
//!
//! For more examples, check out the [`examples` folder on Github](https://github.com/hydro-project/hydroflow/tree/main/hydroflow/examples).

pub mod compiled;
pub mod scheduled;
pub mod util;

#[cfg(feature = "meta")]
pub use dfir_lang as lang;
#[cfg(feature = "python")]
pub use pyo3;
pub use variadics::{self, var_args, var_expr, var_type};
pub use {
    bincode, bytes, futures, itertools, lattices, pusherator, rustc_hash, serde, serde_json, tokio,
    tokio_stream, tokio_util, tracing, web_time,
};

/// `#[macro_use]` automagically brings the declarative macro export to the crate-level.
mod declarative_macro;
#[cfg(feature = "dfir_datalog")]
pub use dfir_datalog::*;
#[cfg(feature = "dfir_macro")]
pub use dfir_macro::{
    dfir_main as main, dfir_parser, dfir_syntax, dfir_syntax_noemit, dfir_test as test,
    monotonic_fn, morphism, DemuxEnum,
};

// TODO(mingwei): Use the [nightly "never" type `!`](https://doc.rust-lang.org/std/primitive.never.html)
/// Stand-in for the [nightly "never" type `!`](https://doc.rust-lang.org/std/primitive.never.html)
pub type Never = std::convert::Infallible;

#[cfg(doctest)]
mod booktest {
    mod surface_ops {
        dfir_macro::surface_booktest_operators!();
    }
}
