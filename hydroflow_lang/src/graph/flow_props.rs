use serde::{Deserialize, Serialize};

/// Stream and lattice properties. Used to determine correctness for scaling transformations.
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct FlowProps {
    /// An abstract token representing the "order" and provenance of a flow.
    ///
    /// TODO(mingwei): may have richer order info later
    pub star_ord: usize,
    /// The lattice flow type (for lattice flows) or `None` for sequential dataflow.
    pub lattice_flow_type: Option<LatticeFlowType>,
}

/// Type of lattice flow.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum LatticeFlowType {
    /// Delta: Elements are (generally) disjoint, each new element represents incremental progress.
    Delta,
    /// Cumulative: Each element must be greater than or equal to the previous. Used for monotonic
    /// functions such as thresholding.
    Cumul,
}
