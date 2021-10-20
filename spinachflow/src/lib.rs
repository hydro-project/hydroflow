#![allow(clippy::redundant_closure)]
#![allow(incomplete_features)]

// #![doc = include_str!("../../README.md")]

#![feature(array_methods)]
#![feature(array_zip)]
#![feature(associated_type_defaults)]
// #![feature(async_stream)]
#![feature(cell_update)]
#![feature(core_intrinsics)]
#![feature(const_type_id)]
#![feature(drain_filter)]
#![feature(generic_associated_types)]
#![feature(slice_as_chunks)]
#![feature(try_blocks)]
#![feature(type_alias_impl_trait)]
#![feature(never_type)]

#![forbid(unsafe_code)]

// Rexports

pub use bincode;
pub use bytes;
pub use futures;
pub use serde;
pub use tokio;
pub use tokio_util;

// Modules

pub mod collections;

pub mod func;

pub mod tag;

pub mod lattice;

pub mod hide;

pub mod op;

pub mod comp;

pub mod metadata;

pub mod tcp_server;

pub mod stream;

