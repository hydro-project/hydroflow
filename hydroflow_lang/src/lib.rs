//! Hydroflow surface syntax

#![warn(missing_docs)]
#![cfg_attr(
    feature = "diagnostics",
    feature(proc_macro_diagnostic, proc_macro_span)
)]
#![allow(clippy::let_and_return)]
#![allow(clippy::explicit_auto_deref)]
pub mod diagnostic;
pub mod graph;
pub mod parse;
pub mod pretty_span;
pub mod process_singletons;
pub mod union_find;
