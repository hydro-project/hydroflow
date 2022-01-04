pub mod context;
pub mod ctx;
pub mod graph;
pub mod graph_demux;
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
pub mod type_list;

pub type SubgraphId = usize;
pub type HandoffId = usize;
pub type StateId = usize;
