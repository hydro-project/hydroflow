use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::rc::Rc;

use hydroflow::lattices::collections::MapMapValues;
use hydroflow_lang::graph::eliminate_extra_unions_tees;
use stageleft::QuotedContext;

use crate::ir::HfPlusLeaf;
use crate::location::{ClusterSpec, LocalDeploy, Location, ProcessSpec};
use crate::{HfCompiled, RuntimeContext};

pub struct FlowBuilder<'a, D: LocalDeploy<'a> + ?Sized> {
    ir_leaves: Rc<RefCell<Vec<HfPlusLeaf>>>,
    nodes: RefCell<Vec<D::Process>>,
    clusters: RefCell<Vec<D::Cluster>>,

    /// Tracks metadata about concrete deployments in this graph, such
    /// as the IDs of each node in a cluster. This is written to
    /// by `ProcessSpec` and `ClusterSpec` and is written to
    /// each instantiated node and cluster via `HfNode::update_meta`.
    meta: RefCell<D::Meta>,

    next_node_id: RefCell<usize>,
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a, D: LocalDeploy<'a>> QuotedContext for FlowBuilder<'a, D> {
    fn create() -> Self {
        FlowBuilder::new()
    }
}

impl<'a, D: LocalDeploy<'a>> FlowBuilder<'a, D> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> FlowBuilder<'a, D> {
        FlowBuilder {
            ir_leaves: Rc::new(RefCell::new(Vec::new())),
            nodes: RefCell::new(Vec::new()),
            clusters: RefCell::new(Vec::new()),
            meta: RefCell::new(Default::default()),
            next_node_id: RefCell::new(0),
            _phantom: PhantomData,
        }
    }

    pub fn extract(self) -> BuiltFlow<'a, D> {
        BuiltFlow {
            ir: self.ir_leaves.borrow().clone(),
            nodes: self.nodes.into_inner(),
            clusters: self.clusters.into_inner(),
            _phantom: PhantomData,
        }
    }

    pub fn ir_leaves(&self) -> &Rc<RefCell<Vec<HfPlusLeaf>>> {
        &self.ir_leaves
    }

    pub fn process(&self, spec: &impl ProcessSpec<'a, D>) -> D::Process {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let node = spec.build(id, self, &mut self.meta.borrow_mut());
        self.nodes.borrow_mut().push(node.clone());

        self.update_metas();

        node
    }

    pub fn cluster(&self, spec: &impl ClusterSpec<'a, D>) -> D::Cluster {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let cluster = spec.build(id, self, &mut self.meta.borrow_mut());
        self.clusters.borrow_mut().push(cluster.clone());

        self.update_metas();

        cluster
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    fn update_metas(&self) {
        self.nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.update_meta(&self.meta.borrow()));
        self.clusters
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.update_meta(&self.meta.borrow()));
    }
}

#[derive(Clone)]
pub struct BuiltFlow<'a, D: LocalDeploy<'a>> {
    pub(crate) ir: Vec<HfPlusLeaf>,
    nodes: Vec<D::Process>,
    clusters: Vec<D::Cluster>,

    _phantom: PhantomData<&'a mut &'a D>,
}

impl<'a, D: LocalDeploy<'a>> BuiltFlow<'a, D> {
    pub fn ir(&self) -> &Vec<HfPlusLeaf> {
        &self.ir
    }

    pub fn optimize_with(
        self,
        f: impl FnOnce(Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf>,
    ) -> BuiltFlow<'a, D> {
        BuiltFlow {
            ir: f(self.ir),
            nodes: self.nodes,
            clusters: self.clusters,
            _phantom: PhantomData,
        }
    }
}

fn build_inner<'a, D: LocalDeploy<'a>>(me: BuiltFlow<'a, D>) -> HfCompiled<'a, D::GraphId> {
    let mut builders = BTreeMap::new();
    let mut built_tees = HashMap::new();
    let mut next_stmt_id = 0;
    for leaf in me.ir {
        leaf.emit(&mut builders, &mut built_tees, &mut next_stmt_id);
    }

    HfCompiled {
        hydroflow_ir: builders.map_values(|v| {
            let (mut flat_graph, _, _) = v.build();
            eliminate_extra_unions_tees(&mut flat_graph);
            flat_graph
        }),
        _phantom: PhantomData,
    }
}

impl<'a, D: LocalDeploy<'a>> BuiltFlow<'a, D> {
    pub fn no_optimize(self) -> HfCompiled<'a, D::GraphId> {
        build_inner(self)
    }

    pub fn optimize_default(self) -> HfCompiled<'a, D::GraphId> {
        self.optimize_with(super::persist_pullup::persist_pullup)
            .no_optimize()
    }
}
