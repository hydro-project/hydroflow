use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};

use serde::{Deserialize, Serialize};

use super::{GraphNodeId, GraphSubgraphId};

#[derive(Default, Serialize, Deserialize)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct SerdeGraph {
    pub nodes: SecondaryMap<GraphNodeId, String>,
    pub edges: SecondaryMap<GraphNodeId, Vec<GraphNodeId>>,
    pub handoffs: SparseSecondaryMap<GraphNodeId, bool>,
    pub subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
}
impl SerdeGraph {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn serde_to_mermaid(&self) -> String {
        let mut output = String::new();
        self.serde_write_mermaid(&mut output).unwrap();
        output
    }

    pub fn serde_write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (subgraph_id, node_ids) in self.subgraph_nodes.iter() {
            writeln!(write, "    subgraph sg_{}", subgraph_id.data().as_ffi())?;
            for &node_id in node_ids.iter() {
                if !self.handoffs.contains_key(node_id) {
                    writeln!(
                        write,
                        r#"        {id:?}["{id:?} <tt>{code}</tt>"]"#,
                        id = node_id.data(),
                        code = self
                            .nodes
                            .get(node_id)
                            .unwrap()
                            .clone()
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\n', "<br>"),
                    )?;
                }
            }
            writeln!(write, "    end")?;
        }
        writeln!(write)?;
        for (node_id, _) in self.nodes.iter() {
            if self.handoffs.contains_key(node_id) {
                writeln!(write, r#"    {:?}{{"handoff"}}"#, node_id.data())?;
            }
        }
        writeln!(write)?;
        for (src, dests) in self.edges.iter() {
            for dst in dests {
                writeln!(write, "    {:?}-->{:?}", src.data(), dst.data())?;
            }
        }
        Ok(())
    }

    pub fn serde_to_dot(&self) -> String {
        let mut output = String::new();
        self.serde_write_dot(&mut output).unwrap();
        output
    }

    pub fn serde_write_dot_edge(
        &self,
        src: GraphNodeId,
        dst: GraphNodeId,
        tab: usize,
        w: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        writeln!(
            w,
            "{:t$}{} -> {}",
            "",
            src.data().as_ffi(),
            dst.data().as_ffi(),
            t = tab,
        )?;
        Ok(())
    }
    pub fn serde_write_dot(&self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "digraph {{")?;
        let mut tab: usize = 2;

        // write out nodes
        writeln!(w, "{:t$}{{", "", t = tab)?;
        tab += 2;
        writeln!(w, "{:t$}node [shape=box]", "", t = tab)?;
        for (node_id, text) in self.nodes.iter() {
            let nm = text.replace('"', "\\\"").replace('\n', "\\l");
            let label = format!("{}", node_id.data().as_ffi());
            write!(
                w,
                "{:t$}{} [label=\"{}{}\"",
                "",
                label,
                nm.clone(),
                match nm.contains("\\l") {
                    // if contains linebreak left-justify by appending another "\\l"
                    true => "\\l",
                    false => "",
                },
                t = tab
            )?;
            if self.handoffs.contains_key(node_id) {
                write!(w, ", shape=diamond")?;
            }
            writeln!(w, "]")?;
        }
        tab -= 2;
        writeln!(w, "{:t$}}}", "", t = tab)?;

        // write out edges per subgraph
        for (sg_id, nodes) in &self.subgraph_nodes {
            // let strt = es.stratum;
            writeln!(
                w,
                "{:t$}subgraph \"cluster stratum {}\" {{",
                "",
                "tbd",
                t = tab,
            )?;
            tab += 2;
            writeln!(w, "{:t$}label = \"Stratum {}\"", "", "tbd", t = tab,)?;
            writeln!(
                w,
                "{:t$}subgraph \"cluster {}\" {{",
                "",
                sg_id.data().as_ffi(),
                t = tab
            )?;
            tab += 2;
            writeln!(w, "{:t$}label = \"{}\"", "", "tbd", t = tab)?;
            let empty = vec![];
            for src in nodes {
                let dests = self.edges.get(*src).unwrap_or(&empty);
                for dst in dests {
                    if !self.handoffs.contains_key(*src) && !self.handoffs.contains_key(*dst) {
                        self.serde_write_dot_edge(*src, *dst, tab, w)?;
                    }
                }
            }
            tab -= 2;
            writeln!(w, "{:t$}}}", "", t = tab)?;
            tab -= 2;
            writeln!(w, "{:t$}}}", "", t = tab)?;
        }

        //write out edges adjacent to handoffs outside the clusters
        for (src, dests) in self.edges.iter() {
            for &dst in dests {
                if self.handoffs.contains_key(src) || self.handoffs.contains_key(dst) {
                    self.serde_write_dot_edge(src, dst, tab, w)?;
                }
            }
        }

        tab -= 2;
        writeln!(w, "{:t$}}}", "", t = tab)?;
        Ok(())
    }
}
