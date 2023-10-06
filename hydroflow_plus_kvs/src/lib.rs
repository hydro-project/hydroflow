#![allow(unused)]

pub(crate) use hydroflow_plus_kvs_macro as __macro;

pub mod __flow {
    include!(concat!(env!("OUT_DIR"), "/lib_pub.rs"));
}

include!(concat!(env!("OUT_DIR"), "/lib.rs"));
