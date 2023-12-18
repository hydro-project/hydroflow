stageleft::stageleft_no_entry_crate!();

mod runtime;
use std::collections::HashMap;

pub use runtime::*;

#[cfg(feature = "deploy")]
mod deploy;

#[cfg(feature = "deploy")]
pub use deploy::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HydroflowPlusMeta {
    pub clusters: HashMap<usize, Vec<usize>>,
}
