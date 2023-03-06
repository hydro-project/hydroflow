use std::fmt::Debug;
use std::iter::FusedIterator;

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};
use syn::spanned::Spanned;
use syn::Ident;

use crate::diagnostic::Diagnostic;
use crate::pretty_span::{PrettyRowCol, PrettySpan};

use super::{
    DiMulGraph, GraphEdgeId, GraphNodeId, Node, OperatorInstance, PartitionedGraph, PortIndexValue,
    HANDOFF_NODE_STR,
};

/// A graph representing a hydroflow dataflow graph before subgraph partitioning, stratification, and handoff insertion.
/// I.e. the graph is a simple "flat" without any subgraph heirarchy.
///
/// May optionally contain handoffs, but in this stage these are transparent and treated like an identity operator.
///
/// Use `Self::into_partitioned_graph()` to convert into a subgraph-partitioned & stratified graph.
#[derive(Debug, Default)]
pub struct FlatGraph {
    /// Each node (operator or handoff).
    nodes: SlotMap<GraphNodeId, Node>,
    /// Instance data corresponding to each operator node.
    operator_instances: SecondaryMap<GraphNodeId, OperatorInstance>,
    /// Graph
    graph: DiMulGraph<GraphNodeId, GraphEdgeId>,
    /// Input and output port for each edge.
    ports: SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,

    /// What variable name each graph node belongs to (if any).
    node_varnames: SparseSecondaryMap<GraphNodeId, Ident>,
}

impl FlatGraph {
    /// Create a new `FlatGraph`, same as `Default::default()`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert a node, assigning the given varname.
    pub fn insert_node(&mut self, node: Node, varname_opt: Option<Ident>) -> GraphNodeId {
        let node_id = self.nodes.insert(node);
        if let Some(varname) = varname_opt {
            self.node_varnames.insert(node_id, varname);
        }
        node_id
    }

    pub fn insert_operator_instance(&mut self, node_id: GraphNodeId, op_inst: OperatorInstance) {
        assert!(matches!(self.nodes.get(node_id), Some(Node::Operator(_))));
        self.operator_instances.insert(node_id, op_inst);
    }

    /// Insert an edge between nodes thru the given ports.
    pub fn insert_edge(
        &mut self,
        src: GraphNodeId,
        src_port: PortIndexValue,
        dst: GraphNodeId,
        dst_port: PortIndexValue,
    ) -> GraphEdgeId {
        let edge_id = self.graph.insert_edge(src, dst);
        self.ports.insert(edge_id, (src_port, dst_port));
        edge_id
    }

    /// Get a node with its operator instance (if applicable).
    pub fn node(&self, node_id: GraphNodeId) -> (&Node, Option<&OperatorInstance>) {
        (&self.nodes[node_id], self.operator_instances.get(node_id))
    }

    /// Iterator over `(GraphNodeId, &Node)` pairs.
    pub fn nodes(&self) -> slotmap::basic::Iter<GraphNodeId, Node> {
        self.nodes.iter()
    }

    /// Get edge: `(src GraphNodeId, src &PortIndexValue, dst GraphNodeId, dst &PortIndexValue))`.
    pub fn edge(
        &self,
        edge_id: GraphEdgeId,
    ) -> (GraphNodeId, &PortIndexValue, GraphNodeId, &PortIndexValue) {
        let (src, dst) = self.graph.edge(edge_id).expect("Edge not found");
        let (src_port, dst_port) = &self.ports[edge_id];
        (src, src_port, dst, dst_port)
    }

    /// Iterator over all edges: `(GraphEdgeId, (src GraphNodeId, src &PortIndexValue, dst GraphNodeId, dst &PortIndexValue))`.
    pub fn edges(
        &self,
    ) -> impl '_
           + Iterator<
        Item = (
            GraphEdgeId,
            (GraphNodeId, &PortIndexValue, GraphNodeId, &PortIndexValue),
        ),
    >
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.edges().map(|(edge_id, (src, dst))| {
            let (src_port, dst_port) = &self.ports[edge_id];
            (edge_id, (src, src_port, dst, dst_port))
        })
    }

    /// Successors, iterator of `(&PortIndexValue, GraphNodeId)` of outgoing edges.
    /// `PortIndexValue` for the port coming out of `src`.
    pub fn successors(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = (GraphEdgeId, &PortIndexValue, GraphNodeId)>
           + DoubleEndedIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph
            .successors(src)
            .map(|(e, v)| (e, &self.ports[e].0, v))
    }

    /// Predecessors, iterator of `(&PortIndexValue, GraphNodeId)` of incoming edges.
    /// `PortIndexValue` for the port going into `dst`.
    pub fn predecessors(
        &self,
        dst: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = (GraphEdgeId, &PortIndexValue, GraphNodeId)>
           + DoubleEndedIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph
            .predecessors(dst)
            .map(|(e, v)| (e, &self.ports[e].1, v))
    }

    /// Degree into a node.
    pub fn degree_in(&self, dst: GraphNodeId) -> usize {
        self.graph.degree_in(dst)
    }

    /// Degree out of a node.
    pub fn degree_out(&self, src: GraphNodeId) -> usize {
        self.graph.degree_out(src)
    }

    /// Convert back into surface syntax.
    pub fn surface_syntax_string(&self) -> String {
        let mut string = String::new();
        self.write_surface_syntax(&mut string).unwrap();
        string
    }

    /// Convert back into surface syntax.
    pub fn write_surface_syntax(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (key, node) in self.nodes.iter() {
            match node {
                Node::Operator(op) => {
                    writeln!(write, "{:?} = {};", key.data(), op.to_token_stream())?;
                }
                Node::Handoff { .. } => unimplemented!("HANDOFF IN FLAT GRAPH."),
            }
        }
        writeln!(write)?;
        for (_e, (src_key, dst_key)) in self.graph.edges() {
            writeln!(write, "{:?} -> {:?};", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

    /// Convert into a [mermaid](https://mermaid-js.github.io/) graph.
    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    /// Convert into a [mermaid](https://mermaid-js.github.io/) graph.
    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => writeln!(
                    write,
                    "    %% {span}\n    {id:?}[\"{row_col} <tt>{code}</tt>\"]",
                    span = PrettySpan(node.span()),
                    id = key.data(),
                    row_col = PrettyRowCol(node.span()),
                    code = operator
                        .to_token_stream()
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;")
                        .replace('\n', "<br>"),
                ),
                Node::Handoff { .. } => {
                    writeln!(write, r#"    {:?}{{"{}"}}"#, key.data(), HANDOFF_NODE_STR)
                }
            }?;
        }
        writeln!(write)?;
        for (_e, (src_key, dst_key)) in self.graph.edges() {
            writeln!(write, "    {:?}-->{:?}", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

    /// Run subgraph partitioning and stratification and convert this graph into a [`PartitionedGraph`].
    #[allow(clippy::result_unit_err)]
    pub fn into_partitioned_graph(self) -> Result<PartitionedGraph, Diagnostic> {
        self.try_into()
    }

    /// TODO(mingwei): remove this, this is temporary for `flat_to_partitioned.rs`.
    pub(super) fn explode(self) -> FlatGraphExploded {
        self.into()
    }
}

/// TODO(mingwei): remove this, this is temporary for `flat_to_partitioned.rs`.
pub struct FlatGraphExploded {
    pub nodes: SlotMap<GraphNodeId, Node>,
    pub operator_instances: SecondaryMap<GraphNodeId, OperatorInstance>,
    pub graph: DiMulGraph<GraphNodeId, GraphEdgeId>,
    pub ports: SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    pub node_varnames: SparseSecondaryMap<GraphNodeId, Ident>,
}
impl From<FlatGraph> for FlatGraphExploded {
    fn from(value: FlatGraph) -> Self {
        let FlatGraph {
            nodes,
            operator_instances,
            graph,
            ports,
            node_varnames,
        } = value;
        Self {
            nodes,
            operator_instances,
            graph,
            ports,
            node_varnames,
        }
    }
}
