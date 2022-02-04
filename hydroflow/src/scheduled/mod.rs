pub mod context;
pub mod graph;
pub mod graph_ext;
pub mod handoff;
#[cfg(feature = "variadic_generics")]
pub mod input;
pub mod net;
pub mod port;
pub mod query;
pub mod reactor;
pub mod state;
pub(crate) mod subgraph;
pub mod type_list;
pub mod util;

pub type SubgraphId = usize;
pub type HandoffId = usize;
pub type StateId = usize;
