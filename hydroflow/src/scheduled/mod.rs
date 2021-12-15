pub mod context;
pub mod ctx;
pub mod graph;
pub mod graph_ext;
pub mod handoff;
#[cfg(feature = "variadic_generics")]
pub mod input;
pub mod net;
pub mod query;
pub mod reactor;
pub mod state;
pub(crate) mod subgraph;
pub mod util;

mod handoff_list;
pub use handoff_list::HandoffList;

pub type SubgraphId = usize;
pub type HandoffId = usize;
pub type StateId = usize;
