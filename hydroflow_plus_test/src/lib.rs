stageleft::stageleft_crate!(hydroflow_plus_test_macro);

#[cfg(stageleft_macro)]
pub(crate) mod cluster;
#[cfg(not(stageleft_macro))]
pub mod cluster;

#[cfg(stageleft_macro)]
pub(crate) mod local;
#[cfg(not(stageleft_macro))]
pub mod local;

#[cfg(stageleft_macro)]
pub(crate) mod distributed;
#[cfg(not(stageleft_macro))]
pub mod distributed;
