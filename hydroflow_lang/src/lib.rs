#![cfg_attr(
    feature = "diagnostics",
    feature(proc_macro_diagnostic, proc_macro_span)
)]
#![feature(iter_intersperse, array_zip)]
#![allow(clippy::let_and_return)]
#![allow(clippy::explicit_auto_deref)]
pub mod diagnostic;
pub mod graph;
pub mod parse;
pub mod pretty_span;
pub mod union_find;

#[doc(hidden)]
pub const __OUT_DIR: &str = std::env!("OUT_DIR");
