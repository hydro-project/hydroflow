#![feature(box_patterns)]

stageleft::stageleft_no_entry_crate!();

pub use hydroflow;
pub use hydroflow::scheduled::graph::Hydroflow;
pub use stageleft::*;

pub mod runtime_support {
    pub use bincode;
}

pub mod runtime_context;
pub use runtime_context::RuntimeContext;

pub mod boundedness;
pub use boundedness::{Bounded, Unbounded};

pub mod stream;
pub use stream::Stream;

pub mod singleton;
pub use singleton::Singleton;

pub mod optional;
pub use optional::Optional;

pub mod location;
pub use location::{Cluster, ClusterId, Location, Process, Tick};

pub mod deploy;

pub mod cycle;

pub mod builder;
pub use builder::FlowBuilder;

pub mod ir;

pub mod rewrites;

mod staging_util;

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    #[ctor::ctor]
    fn init() {
        crate::deploy::init_test();
    }
}
