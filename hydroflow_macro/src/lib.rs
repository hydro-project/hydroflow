#![feature(proc_macro_diagnostic, proc_macro_span)]

use std::collections::{HashMap, HashSet};

use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use slotmap::{new_key_type, Key, SecondaryMap, SlotMap};
use syn::punctuated::Pair;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, LitInt};

mod parse;
use parse::{HfCode, HfStatement, Operator, Pipeline};
use union_find::UnionFind;

mod union_find;

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);
    // // input.into_token_stream().into()

    let mut graph = Graph::from_hfcode(input).unwrap(/* TODO(mingwei) */);
    graph.validate_operators();
    graph.identify_subgraphs();

    // let debug = format!("{:#?}", graph);
    // let mut debug = String::new();
    // graph.write_graph(&mut debug).unwrap();
    let mut debug = String::new();
    graph.write_mermaid_subgraphs(&mut debug).unwrap();

    let lit = Literal::string(&*debug);

    quote! { println!("{}", #lit); }.into()
}

new_key_type! { struct NodeId; }
new_key_type! { struct SubgraphId; }

type EdgePort = (NodeId, LitInt);
type EdgePortRef<'a> = (NodeId, &'a LitInt);

#[derive(Debug, Default)]
struct Graph {
    operators: SlotMap<NodeId, NodeInfo>,
    names: HashMap<Ident, Ports>,
    /// Equivalent to SparseSecondaryMap<...>.
    subgraphs: HashMap<NodeId, Vec<NodeId>>,
}
impl Graph {
    pub fn from_hfcode(input: HfCode) -> Result<Self, ()> {
        let mut graph = Self::default();

        for stmt in input.statements {
            graph.add_statement(stmt);
        }

        Ok(graph)
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
                                    &*self.operators[out].succs.len().to_string(),
                                    curr_arrow.arrow.span(),
                                )
                            });
                            let dst_port = curr_arrow.dst.map(|x| x.index).unwrap_or_else(|| {
                                LitInt::new(
                                    &*self.operators[inn].preds.len().to_string(),
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

                                if let Some((old_a, _)) =
                                    self.operators[out].succs.remove_entry(&src_a)
                                {
                                    emit_conflict("Output", &old_a, &src_a);
                                }
                                self.operators[out].succs.insert(src_a, (inn, dst_a));

                                if let Some((old_b, _)) =
                                    self.operators[inn].preds.remove_entry(&dst_b)
                                {
                                    emit_conflict("Input", &old_b, &dst_b);
                                }
                                self.operators[inn].preds.insert(dst_b, (out, src_b));
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
                let (preds, succs) = Default::default();
                let port = self.operators.insert(NodeInfo {
                    node: Node::Operator(operator),
                    preds,
                    succs,
                    subgraph_id: None,
                });
                Ports {
                    inn: Some(port),
                    out: Some(port),
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

        for node_info in self.operators.values() {
            match &node_info.node {
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

                    if !inn_allowed.contains(&node_info.preds.len()) {
                        operator
                            .span()
                            .unwrap()
                            .error(format!(
                        "`{}` has invalid number of inputs: {}. Allowed is between {:?} and {:?}.",
                        op_name,
                        &node_info.preds.len(),
                        inn_allowed.start_bound(),
                        inn_allowed.end_bound()
                    ))
                            .emit();
                    }
                    if !out_allowed.contains(&node_info.succs.len()) {
                        operator
                            .span()
                            .unwrap()
                            .error(format!(
                        "`{}` has invalid number of outputs: {}. Allowed is between {:?} and {:?}.",
                        op_name,
                        &node_info.succs.len(),
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
        self.operators.iter().flat_map(|(src, node_info)| {
            node_info
                .succs
                .iter()
                .map(move |(src_idx, (dst, dst_idx))| ((src, src_idx), (*dst, dst_idx)))
        })
    }

    pub fn identify_subgraphs(&mut self) {
        // Pull (green)
        // Push (blue)
        // Handoff (red) -- not a color for operators, inserted between subgraphs.
        // Computation (yellow)
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        enum Color {
            Pull,
            Push,
            Comp,
            Hoff,
        }
        fn op_color(node_info: &NodeInfo) -> Option<Color> {
            match &node_info.node {
                Node::Operator(_) => match (1 < node_info.preds.len(), 1 < node_info.succs.len()) {
                    (true, true) => Some(Color::Comp),
                    (true, false) => Some(Color::Pull),
                    (false, true) => Some(Color::Push),
                    (false, false) => {
                        match (node_info.preds.is_empty(), node_info.succs.is_empty()) {
                            (true, false) => Some(Color::Pull),
                            (false, true) => Some(Color::Push),
                            _same => None,
                        }
                    }
                },
                Node::Handoff => Some(Color::Hoff),
            }
        }

        // Algorithm:
        // 1. Each node begins as its own subgraph.
        // 2. Collect edges. Sort so edges which should not be split across a handoff come first.
        // 3. For each edge, try to join `(to, from)` into the same subgraph.

        let mut node_color: SecondaryMap<NodeId, Option<Color>> = self
            .operators
            .iter()
            .map(|(id, node_info)| (id, op_color(node_info)))
            .collect();
        let mut node_union: UnionFind<NodeId> = self.operators.keys().collect();
        // All edges which belong to a single subgraph. Other & self-edges become handoffs.
        let mut subgraph_edges: HashSet<(EdgePortRef, EdgePortRef)> = Default::default();

        // Sort edges here (for now, no sort/priority).
        loop {
            let mut updated = false;
            for ((src, src_idx), (dst, dst_idx)) in self.edges() {
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

        // Insert handoffs between subgraphs (or on subgraph self-edges).
        let handoff_edges: Vec<_> = self
            .edges()
            .filter(|edge| !subgraph_edges.contains(edge)) // Subgraph edges are not handoffs.
            .filter(|&((src, _src_idx), (dst, _dst_idx))| {
                !matches!(self.operators[src].node, Node::Handoff)
                    && !matches!(self.operators[dst].node, Node::Handoff)
            }) // Already has a handoff.
            .map(|((src, src_idx), (dst, dst_idx))| {
                ((src, src_idx.clone()), (dst, dst_idx.clone()))
            })
            .collect();

        for ((src, src_idx), (dst, dst_idx)) in handoff_edges {
            let handoff_nodeid = self.operators.insert(NodeInfo {
                node: Node::Handoff,
                preds: Default::default(),
                succs: Default::default(),
                subgraph_id: None,
            });

            // Insert forward edge.
            let (dst2, dst_idx2) = self.operators[src]
                .succs
                .insert(
                    src_idx,
                    (handoff_nodeid, LitInt::new("0", Span::call_site())),
                )
                .expect("Forward edge disappeared!");

            // Insert back edge.
            let (src2, src_idx2) = self.operators[dst]
                .preds
                .insert(
                    dst_idx,
                    (handoff_nodeid, LitInt::new("0", Span::call_site())),
                )
                .expect("Back edge disappeared!");

            self.operators[handoff_nodeid]
                .preds
                .insert(LitInt::new("0", Span::call_site()), (src2, src_idx2));
            self.operators[handoff_nodeid]
                .succs
                .insert(LitInt::new("0", Span::call_site()), (dst2, dst_idx2));
        }

        // Set `subgraph_id`s inside node infos.
        for (key, node_info) in self.operators.iter_mut() {
            let subgraph_key = node_union.find(key);
            node_info.subgraph_id = Some(subgraph_key);
            self.subgraphs.entry(subgraph_key).or_default().push(key);
        }
    }

    pub fn subgraphs(&self) -> std::collections::hash_map::Iter<'_, NodeId, Vec<NodeId>> {
        self.subgraphs.iter()
    }

    #[allow(dead_code)]
    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    #[allow(dead_code)]
    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (key, node_info) in self.operators.iter() {
            match &node_info.node {
                Node::Operator(operator) => writeln!(
                    write,
                    r#"    {}["{} {:?}"]"#,
                    key.data().as_ffi(),
                    operator
                        .to_token_stream()
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;"),
                    node_info.subgraph_id.map(|id| id.data().as_ffi()),
                ),
                Node::Handoff => writeln!(write, r#"    {}{{"handoff"}}"#, key.data().as_ffi()),
            }?;
        }
        writeln!(write)?;
        for (src_key, op) in self.operators.iter() {
            for (_src_port, (dst_key, _dst_port)) in op.succs.iter() {
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

    pub fn write_mermaid_subgraphs(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (subgraph_id, node_ids) in self.subgraphs() {
            writeln!(write, "    subgraph sg_{}", subgraph_id.data().as_ffi())?;
            for &node_id in node_ids.iter() {
                let node_info = &self.operators[node_id];
                match &node_info.node {
                    Node::Operator(operator) => {
                        writeln!(
                            write,
                            r#"        {}["{}"]"#,
                            node_id.data().as_ffi(),
                            operator
                                .to_token_stream()
                                .to_string()
                                .replace('&', "&amp;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;")
                                .replace('"', "&quot;"),
                        )?;
                    }
                    Node::Handoff => {
                        // writeln!(write, r#"        {}{{"handoff"}}"#, node_id.data().as_ffi())
                    }
                }
            }
            writeln!(write, "    end")?;
        }
        writeln!(write)?;
        for (node_id, node_info) in self.operators.iter() {
            if matches!(node_info.node, Node::Handoff) {
                writeln!(write, r#"    {}{{"handoff"}}"#, node_id.data().as_ffi())?;
            }
        }
        writeln!(write)?;
        for ((src, _src_idx), (dst, _dst_idx)) in self.edges() {
            writeln!(
                write,
                "    {}-->{}",
                src.data().as_ffi(),
                dst.data().as_ffi()
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct Ports {
    inn: Option<NodeId>,
    out: Option<NodeId>,
}

enum Node {
    Operator(Operator),
    Handoff,
}
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operator(operator) => {
                write!(f, "Node::Operator({} span)", PrettySpan(operator.span()))
            }
            Self::Handoff => write!(f, "Node::Handoff"),
        }
    }
}

struct NodeInfo {
    node: Node,
    preds: HashMap<LitInt, EdgePort>,
    succs: HashMap<LitInt, EdgePort>,

    /// Which subgraph this operator belongs to (if determined).
    subgraph_id: Option<NodeId>,
    // color: Option<Color>,
}
impl std::fmt::Debug for NodeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeInfo")
            .field("operator", &self.node)
            .field("preds", &self.preds)
            .field("succs", &self.succs)
            .finish()
    }
}

/// Helper struct which displays the span as `path:row:col` for human reading/IDE linking.
/// Example: `hydroflow\tests\surface_syntax.rs:42:18`.
struct PrettySpan(Span);
impl std::fmt::Display for PrettySpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.0.unwrap();
        write!(
            f,
            "{}:{}:{}",
            span.source_file().path().display(),
            span.start().line,
            span.start().column
        )
    }
}
