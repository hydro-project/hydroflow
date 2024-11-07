use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use stageleft::*;

use crate::ir::HfPlusLeaf;
use crate::location::{Cluster, ExternalProcess, Process};
use crate::RuntimeContext;

pub mod built;
pub mod compiled;
pub mod deploy;

pub struct FlowStateInner {
    /// Tracks the leaves of the dataflow IR. This is referenced by
    /// `Stream` and `HfCycle` to build the IR. The inner option will
    /// be set to `None` when this builder is finalized.
    pub(crate) leaves: Option<Vec<HfPlusLeaf>>,

    /// Counter for generating unique external output identifiers.
    pub(crate) next_external_out: usize,

    /// Counters for generating identifiers for cycles.
    pub(crate) cycle_counts: HashMap<usize, usize>,

    /// Counters for clock IDs.
    pub(crate) next_clock_id: usize,
}

pub type FlowState = Rc<RefCell<FlowStateInner>>;

pub const FLOW_USED_MESSAGE: &str = "Attempted to add a leaf to a flow that has already been finalized. No leaves can be added after the flow has been compiled.";

pub struct FlowBuilder<'a> {
    flow_state: FlowState,
    nodes: RefCell<Vec<usize>>,
    clusters: RefCell<Vec<usize>>,

    next_node_id: RefCell<usize>,

    /// Tracks whether this flow has been finalized; it is an error to
    /// drop without finalizing.
    finalized: bool,

    /// 'a on a FlowBuilder is used to ensure that staged code does not
    /// capture more data that it is allowed to; 'a is generated at the
    /// entrypoint of the staged code and we keep it invariant here
    /// to enforce the appropriate constraints
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl Drop for FlowBuilder<'_> {
    fn drop(&mut self) {
        if !self.finalized {
            panic!("Dropped FlowBuilder without finalizing, you may have forgotten to call `with_default_optimize`, `optimize_with`, or `finalize`.");
        }
    }
}

impl QuotedContext for FlowBuilder<'_> {
    fn create() -> Self {
        FlowBuilder::new()
    }
}

impl<'a> FlowBuilder<'a> {
    #[expect(
        clippy::new_without_default,
        reason = "call `new` explicitly, not `default`"
    )]
    pub fn new() -> FlowBuilder<'a> {
        FlowBuilder {
            flow_state: Rc::new(RefCell::new(FlowStateInner {
                leaves: Some(vec![]),
                next_external_out: 0,
                cycle_counts: HashMap::new(),
                next_clock_id: 0,
            })),
            nodes: RefCell::new(vec![]),
            clusters: RefCell::new(vec![]),
            next_node_id: RefCell::new(0),
            finalized: false,
            _phantom: PhantomData,
        }
    }

    pub fn finalize(mut self) -> built::BuiltFlow<'a> {
        self.finalized = true;

        built::BuiltFlow {
            ir: self.flow_state.borrow_mut().leaves.take().unwrap(),
            processes: self.nodes.replace(vec![]),
            clusters: self.clusters.replace(vec![]),
            used: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> built::BuiltFlow<'a> {
        self.finalize().with_default_optimize()
    }

    pub fn optimize_with(
        self,
        f: impl FnOnce(Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf>,
    ) -> built::BuiltFlow<'a> {
        self.finalize().optimize_with(f)
    }

    pub fn flow_state(&self) -> &FlowState {
        &self.flow_state
    }

    pub fn process<P>(&self) -> Process<'a, P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.nodes.borrow_mut().push(id);

        Process {
            id,
            flow_state: self.flow_state().clone(),
            _phantom: PhantomData,
        }
    }

    pub fn external_process<P>(&self) -> ExternalProcess<'a, P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.nodes.borrow_mut().push(id);

        ExternalProcess {
            id,
            flow_state: self.flow_state().clone(),
            _phantom: PhantomData,
        }
    }

    pub fn cluster<C>(&self) -> Cluster<'a, C> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.clusters.borrow_mut().push(id);

        Cluster {
            id,
            flow_state: self.flow_state().clone(),
            _phantom: PhantomData,
        }
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext::new()
    }
}
