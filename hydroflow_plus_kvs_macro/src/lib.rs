#![allow(unused)]

#[doc(hidden)]
pub(crate) use hydroflow_plus_kvs_flow as __flow;

include!(concat!(env!("OUT_DIR"), "/lib.rs"));
