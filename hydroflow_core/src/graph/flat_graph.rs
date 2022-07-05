use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::punctuated::Pair;
use syn::spanned::Spanned;
use syn::{Ident, LitInt};

use crate::parse::{HfCode, HfStatement, Pipeline};
use crate::pretty_span::PrettySpan;
use crate::union_find::UnionFind;

use super::partitioned_graph::PartitionedGraph;
use super::{EdgePort, EdgePortRef, Node, NodeId, SubgraphId};

#[derive(Debug, Default)]
pub struct FlatGraph {
    nodes: SlotMap<NodeId, Node>,
    preds: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>>,
    succs: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>>,
    names: HashMap<Ident, Ports>,
}
impl FlatGraph {
    // TODO(mingwei): better error/diagnostic handling.
    pub fn from_hfcode(input: HfCode) -> Self {
        let mut graph = Self::default();

        for stmt in input.statements {
            graph.add_statement(stmt);
        }

        graph
    }

    fn add_statement(&mut self, stmt: HfStatement) {
        match stmt {
            HfStatement::Named(named) => {
                let ports = self.add_pipeline(named.pipeline);
                // if let Some((old_name, _)) = self.names.remove_entry(&named.name) {
                //     old_name.span().unwrap().warning(format!("`{}` is shadowed"))
                // }
                self.names.insert(named.name, ports);
            }
            HfStatement::Pipeline(pipeline) => {
                self.add_pipeline(pipeline);
            }
        }
    }

    fn add_pipeline(&mut self, pipeline: Pipeline) -> Ports {
        match pipeline {
            Pipeline::Chain(chain_pipeline) => {
                // Handle chain pipelines as follows:
                let output = chain_pipeline
                    .elems
                    .into_pairs()
                    .map(Pair::into_tuple)
                    // 1. Resolve all the nested pipelines in first stage (collect into Vec before continuing, for ownership).
                    .map(|(pipeline, arrow)| (self.add_pipeline(pipeline), arrow))
                    .collect::<Vec<_>>()
                    .into_iter()
                    // 2. Iterate each element in pairs via `.reduce()` and combine them into the next pipeline.
                    // Essentially, treats the arrows as a left-associative binary operation (not that the direction really matters).
                    // `curr_ports: Ports` tracks the current input/output operators/ports in the graph.
                    .reduce(|(curr_ports, curr_arrow), (next_ports, next_arrow)| {
                        let curr_arrow =
                            curr_arrow.expect("Cannot have missing intermediate arrow");

                        if let (Some(out), Some(inn)) = (curr_ports.out, next_ports.inn) {
                            let src_port = curr_arrow.src.map(|x| x.index).unwrap_or_else(|| {
                                LitInt::new(
                                    &*self.succs[out].len().to_string(),
                                    curr_arrow.arrow.span(),
                                )
                            });
                            let dst_port = curr_arrow.dst.map(|x| x.index).unwrap_or_else(|| {
                                LitInt::new(
                                    &*self.preds[inn].len().to_string(),
                                    curr_arrow.arrow.span(),
                                )
                            });

                            {
                                /// Helper to emit conflicts when a port is overwritten.
                                fn emit_conflict(inout: &str, old: &LitInt, new: &LitInt) {
                                    old.span()
                                        .unwrap()
                                        .error(format!(
                                            "{} connection conflicts with below ({})",
                                            inout,
                                            PrettySpan(new.span()),
                                        ))
                                        .emit();
                                    new.span()
                                        .unwrap()
                                        .error(format!(
                                            "{} connection conflicts with above ({})",
                                            inout,
                                            PrettySpan(old.span()),
                                        ))
                                        .emit();
                                }

                                // Clone, one for `succs` and one for `preds`.
                                let (src_a, src_b) = (src_port.clone(), src_port);
                                let (dst_a, dst_b) = (dst_port.clone(), dst_port);

                                if let Some((old_a, _)) = self.succs[out].remove_entry(&src_a) {
                                    emit_conflict("Output", &old_a, &src_a);
                                }
                                self.succs[out].insert(src_a, (inn, dst_a));

                                if let Some((old_b, _)) = self.preds[inn].remove_entry(&dst_b) {
                                    emit_conflict("Input", &old_b, &dst_b);
                                }
                                self.preds[inn].insert(dst_b, (out, src_b));
                            }
                        }

                        let ports = Ports {
                            inn: curr_ports.inn,
                            out: next_ports.out,
                        };
                        (ports, next_arrow)
                    });

                output.map(|(ports, _arrow)| ports).unwrap_or(Ports {
                    inn: None,
                    out: None,
                })
            }
            Pipeline::Name(ident) => self.names.get(&ident).copied().unwrap_or_else(|| {
                ident
                    .span()
                    .unwrap()
                    .error(format!("Cannot find name `{}`", ident))
                    .emit();
                Ports {
                    inn: None,
                    out: None,
                }
            }),
            Pipeline::Operator(operator) => {
                let key = self.nodes.insert(Node::Operator(operator));
                self.preds.insert(key, Default::default());
                self.succs.insert(key, Default::default());
                Ports {
                    inn: Some(key),
                    out: Some(key),
                }
            }
        }
    }

    /// Validates that operators have valid number of inputs and outputs.
    /// (Emits error messages on span).
    /// TODO(mingwei): Clean this up, make it do more than just arity.
    pub fn validate_operators(&self) {
        use std::ops::{Bound, RangeBounds};
        trait RangeTrait<T>
        where
            T: ?Sized,
        {
            fn start_bound(&self) -> Bound<&T>;
            fn end_bound(&self) -> Bound<&T>;
            fn contains(&self, item: &T) -> bool
            where
                T: PartialOrd<T>;
        }
        impl<R, T> RangeTrait<T> for R
        where
            R: RangeBounds<T>,
        {
            fn start_bound(&self) -> Bound<&T> {
                self.start_bound()
            }

            fn end_bound(&self) -> Bound<&T> {
                self.end_bound()
            }

            fn contains(&self, item: &T) -> bool
            where
                T: PartialOrd<T>,
            {
                self.contains(item)
            }
        }

        for (node_key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => {
                    let op_name = &*operator.path.to_token_stream().to_string();
                    let (inn_allowed, out_allowed): (
                        &dyn RangeTrait<usize>,
                        &dyn RangeTrait<usize>,
                    ) = match op_name {
                        "merge" => (&(2..), &(1..=1)),
                        "join" => (&(2..=2), &(1..=1)),
                        "tee" => (&(1..=1), &(2..)),
                        "map" | "dedup" => (&(1..=1), &(1..=1)),
                        "input" | "seed" => (&(0..=0), &(1..=1)),
                        "for_each" => (&(1..=1), &(0..=0)),
                        unknown => {
                            operator
                                .path
                                .span()
                                .unwrap()
                                .error(format!("Unknown operator `{}`", unknown))
                                .emit();
                            (&(..), &(..))
                        }
                    };

                    let inn_degree = self.preds[node_key].len();
                    if !inn_allowed.contains(&inn_degree) {
                        operator
                            .span()
                            .unwrap()
                            .error(format!(
                        "`{}` has invalid number of inputs: {}. Allowed is between {:?} and {:?}.",
                        op_name,
                        inn_degree,
                        inn_allowed.start_bound(),
                        inn_allowed.end_bound()
                    ))
                            .emit();
                    }

                    let out_degree = self.succs[node_key].len();
                    if !out_allowed.contains(&out_degree) {
                        operator
                            .span()
                            .unwrap()
                            .error(format!(
                        "`{}` has invalid number of outputs: {}. Allowed is between {:?} and {:?}.",
                        op_name,
                        out_degree,
                        out_allowed.start_bound(),
                        out_allowed.end_bound()
                    ))
                            .emit();
                    }
                }
                Node::Handoff => todo!("Node::Handoff"),
            }
        }
    }

    pub fn edges(&self) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
        Self::edges_helper(&self.succs)
    }
    fn edges_helper(
        succs: &SecondaryMap<NodeId, HashMap<LitInt, (NodeId, LitInt)>>,
    ) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
        succs.iter().flat_map(|(src, succs)| {
            succs
                .iter()
                .map(move |(src_idx, (dst, dst_idx))| ((src, src_idx), (*dst, dst_idx)))
        })
    }

    fn op_color(&self, node_key: NodeId) -> Option<Color> {
        let inn_degree = self.preds[node_key].len();
        let out_degree = self.succs[node_key].len();
        match &self.nodes[node_key] {
            Node::Operator(_) => match (1 < inn_degree, 1 < out_degree) {
                (true, true) => Some(Color::Comp),
                (true, false) => Some(Color::Pull),
                (false, true) => Some(Color::Push),
                (false, false) => match (inn_degree, out_degree) {
                    (0, _) => Some(Color::Pull),
                    (_, 0) => Some(Color::Push),
                    _same => None,
                },
            },
            Node::Handoff => Some(Color::Hoff),
        }
    }

    pub fn into_partitioned_graph(mut self) -> PartitionedGraph {
        // Algorithm:
        // 1. Each node begins as its own subgraph.
        // 2. Collect edges. Sort so edges which should not be split across a handoff come first.
        // 3. For each edge, try to join `(to, from)` into the same subgraph.

        let mut node_color: SecondaryMap<NodeId, Option<Color>> = self
            .nodes
            .keys()
            .map(|id| (id, self.op_color(id)))
            .collect();
        let mut node_union: UnionFind<NodeId> = UnionFind::with_capacity(self.nodes.len());
        // All edges which belong to a single subgraph. Other & self-edges become handoffs.
        let mut subgraph_edges: HashSet<(EdgePortRef, EdgePortRef)> = Default::default();

        // Sort edges here (for now, no sort/priority).
        loop {
            let mut updated = false;
            for ((src, src_idx), (dst, dst_idx)) in Self::edges_helper(&self.succs) {
                if node_union.same_set(src, dst) {
                    // Note this might be triggered even if the edge (src, dst) is not in the subgraph.
                    // This prevents self-loops. Handoffs needed to break self loops.
                    continue;
                }

                // Set `src` or `dst` color if `None` based on the other (if possible):
                // Pull -> Pull
                // Push -> Push
                // Pull -> [Comp] -> Push
                // Push -> [Hoff] -> Pull
                match (node_color[src], node_color[dst]) {
                    (Some(_), Some(_)) => (),
                    (None, None) => (),
                    (None, Some(dst_color)) => {
                        node_color[src] = Some(match dst_color {
                            Color::Comp => Color::Pull,
                            Color::Hoff => Color::Push,
                            pull_or_push => pull_or_push,
                        });
                        updated = true;
                    }
                    (Some(src_color), None) => {
                        node_color[dst] = Some(match src_color {
                            Color::Comp => Color::Push,
                            Color::Hoff => Color::Pull,
                            pull_or_push => pull_or_push,
                        });
                        updated = true;
                    }
                }

                // If SRC and DST can be in the same subgraph.
                let can_connect = match (node_color[src], node_color[dst]) {
                    (Some(Color::Pull), Some(Color::Pull)) => true,
                    (Some(Color::Pull), Some(Color::Comp)) => true,
                    (Some(Color::Pull), Some(Color::Push)) => true,

                    (Some(Color::Comp | Color::Push), Some(Color::Pull)) => false,
                    (Some(Color::Comp | Color::Push), Some(Color::Comp)) => false,
                    (Some(Color::Comp | Color::Push), Some(Color::Push)) => true,

                    // Handoffs are not part of subgraphs.
                    (Some(Color::Hoff), Some(_)) => false,
                    (Some(_), Some(Color::Hoff)) => false,

                    // Linear chain.
                    (None, None) => true,

                    _some_none => unreachable!(),
                };
                if can_connect {
                    node_union.union(src, dst);
                    subgraph_edges.insert(((src, src_idx), (dst, dst_idx)));
                    updated = true;
                }
            }
            if !updated {
                break;
            }
        }

        let mut new_preds: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>> =
            self.nodes.keys().map(|k| (k, Default::default())).collect();
        let mut new_succs: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>> =
            self.nodes.keys().map(|k| (k, Default::default())).collect();

        // Copy over edges, inserting handoffs between subgraphs (or on subgraph self-edges) when needed.
        for edge in Self::edges_helper(&self.succs) {
            let is_subgraph_edge = subgraph_edges.contains(&edge); // Internal subgraph edges are not handoffs.
            let ((src, src_idx), (dst, dst_idx)) = edge;

            // Already has a handoff, no need to insert one.
            if is_subgraph_edge
                || matches!(self.nodes[src], Node::Handoff)
                || matches!(self.nodes[dst], Node::Handoff)
            {
                new_preds[dst].insert(dst_idx.clone(), (src, src_idx.clone()));
                new_succs[src].insert(src_idx.clone(), (dst, dst_idx.clone()));
            } else {
                // Needs handoff inserted.
                // A -> H -> Z
                let hoff_id = self.nodes.insert(Node::Handoff);
                new_preds.insert(hoff_id, Default::default());
                new_succs.insert(hoff_id, Default::default());

                // A -> H.
                new_succs[src].insert(
                    src_idx.clone(),
                    (hoff_id, LitInt::new("0", Span::call_site())),
                );
                // A <- H.
                new_preds[hoff_id]
                    .insert(LitInt::new("0", Span::call_site()), (src, src_idx.clone()));
                // H <- Z.
                new_preds[dst].insert(
                    dst_idx.clone(),
                    (hoff_id, LitInt::new("0", Span::call_site())),
                );
                // H -> Z.
                new_succs[hoff_id]
                    .insert(LitInt::new("0", Span::call_site()), (dst, dst_idx.clone()));
            }
        }

        // Mapping from representative `NodeId` to the `SubgraphId`.
        let mut node_to_subgraph = HashMap::new();
        // List of nodes in each `SubgraphId`.
        let mut subgraph_nodes: SlotMap<SubgraphId, Vec<NodeId>> =
            SlotMap::with_capacity_and_key(self.nodes.len());
        // `SubgraphId` for each `NodeId`.
        let node_subgraph = self
            .nodes
            .iter()
            .filter_map(|(node_id, node)| match node {
                Node::Operator(_op) => {
                    let repr_id = node_union.find(node_id);
                    let subgraph_id = *node_to_subgraph
                        .entry(repr_id)
                        .or_insert_with(|| subgraph_nodes.insert(Default::default()));
                    subgraph_nodes[subgraph_id].push(node_id);
                    Some((node_id, subgraph_id))
                }
                Node::Handoff => None,
            })
            .collect();

        PartitionedGraph {
            nodes: self.nodes,
            preds: new_preds,
            succs: new_succs,
            node_subgraph,
            subgraph_nodes,
        }
    }

    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    #[allow(dead_code)]
    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => writeln!(
                    write,
                    r#"    {}["{}"]"#,
                    key.data().as_ffi(),
                    operator
                        .to_token_stream()
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;"),
                ),
                Node::Handoff => writeln!(write, r#"    {}{{"handoff"}}"#, key.data().as_ffi()),
            }?;
        }
        writeln!(write)?;
        for (src_key, _op) in self.nodes.iter() {
            for (_src_port, (dst_key, _dst_port)) in self.succs[src_key].iter() {
                writeln!(
                    write,
                    "    {}-->{}",
                    src_key.data().as_ffi(),
                    dst_key.data().as_ffi()
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct Ports {
    inn: Option<NodeId>,
    out: Option<NodeId>,
}

/// Pull (green)
/// Push (blue)
/// Handoff (red) -- not a color for operators, inserted between subgraphs.
/// Computation (yellow)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Color {
    Pull,
    Push,
    Comp,
    Hoff,
}
