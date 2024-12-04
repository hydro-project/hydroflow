stageleft::stageleft_no_entry_crate!();

pub mod cluster;
pub mod distributed;

#[doc(hidden)]
#[stageleft::runtime]
mod docs {
    #[doc = include_str!("../../docs/docs/hydroflow_plus/consistency.md")]
    mod consistency {}
}
