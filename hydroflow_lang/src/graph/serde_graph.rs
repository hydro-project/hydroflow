use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};

use serde::{Deserialize, Serialize};

use super::{Color, GraphNodeId, GraphSubgraphId};

use regex::Regex;

#[derive(Default, Serialize, Deserialize)]
pub struct SerdeEdge {
    pub src: GraphNodeId,
    pub dst: GraphNodeId,
    pub blocking: bool,
    pub label: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct SerdeGraph {
    // TODO(jmh): this structure has no way to maintain the index/order of incoming edges
    pub nodes: SecondaryMap<GraphNodeId, String>,
    pub edges: SecondaryMap<GraphNodeId, Vec<SerdeEdge>>,
    pub handoffs: SparseSecondaryMap<GraphNodeId, bool>,
    pub subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    pub subgraph_stratum: SecondaryMap<GraphSubgraphId, usize>,
}
impl SerdeGraph {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn to_mermaid(&self) -> String {
        let mut output = String::new();
        self.write_mermaid(&mut output).unwrap();
        output
    }

    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        let re = Regex::new(r"\[(Push|Pull|Hoff)\]").unwrap();

        for (subgraph_id, node_ids) in self.subgraph_nodes.iter() {
            let stratum = self.subgraph_stratum.get(subgraph_id);
            writeln!(
                write,
                "    subgraph \"sg_{:?} stratum {:?}\"",
                subgraph_id.data(),
                stratum.unwrap()
            )?;
            for &node_id in node_ids.iter() {
                if !self.handoffs.contains_key(node_id) {
                    let mode_str = match re.captures(self.nodes.get(node_id).unwrap()) {
                        Some(caps) => {
                            let mode_match = caps.get(1);
                            if let Some(mode_str) = mode_match {
                                mode_str.as_str()
                            } else {
                                ""
                            }
                        }
                        None => "",
                    };
                    let mode = match mode_str {
                        "Push" => Color::Push,
                        "Pull" => Color::Pull,
                        _ => Color::Hoff,
                    };

                    let label = format!(
                        // write,
                        r#"        {id:?}{lbracket}"({id:?}) <tt>{code}</tt>"{rbracket}"#,
                        id = node_id.data(),
                        lbracket = match mode {
                            Color::Push => r"[\",
                            Color::Pull => r"[/",
                            _ => "[",
                        },
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
                        rbracket = match mode {
                            Color::Push => r"/]",
                            Color::Pull => r"\]",
                            _ => "]",
                        }
                    );
                    let label = re.replace(label.as_str(), "");
                    writeln!(write, "{}", label)?;
                }
            }
            writeln!(write, "    end")?;
        }
        writeln!(write)?;
        for node_id in self.nodes.keys() {
            if self.handoffs.contains_key(node_id) {
                writeln!(write, r#"    {:?}{{"handoff"}}"#, node_id.data())?;
            }
        }
        writeln!(write)?;
        for (src, dests) in self.edges.iter() {
            for edge in dests {
                writeln!(
                    write,
                    "{:?}{}{}{:?}",
                    src.data(),
                    if let Some(label) = &edge.label {
                        if edge.blocking {
                            format!("== {}", label)
                        } else {
                            format!("-- {}", label)
                        }
                    } else {
                        "".to_string()
                    },
                    if edge.blocking { "===o" } else { "--->" },
                    edge.dst.data()
                )?;
            }
        }
        Ok(())
    }

    pub fn to_dot(&self) -> String {
        let mut output = String::new();
        self.write_dot(&mut output).unwrap();
        output
    }

    pub fn write_dot(&self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "digraph {{")?;
        let mut tab: usize = 2;

        fn write_dot_edge(
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
                nm,
                // if contains linebreak left-justify by appending another "\\l"
                if nm.contains("\\l") { "\\l" } else { "" },
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
        for (sg_id, nodes) in self.subgraph_nodes.iter() {
            // let strt = es.stratum;
            writeln!(
                w,
                "{:t$}subgraph \"cluster stratum NUMBER UNKNOWN\" {{",
                "",
                t = tab,
            )?;
            tab += 2;
            writeln!(w, "{:t$}label = \"Stratum NUMBER UNKNOWN\"", "", t = tab,)?;
            writeln!(
                w,
                "{:t$}subgraph \"cluster {}\" {{",
                "",
                sg_id.data().as_ffi(),
                t = tab
            )?;
            tab += 2;
            writeln!(
                w,
                "{:t$}label = \"sg_{}\"",
                "",
                sg_id.data().as_ffi(),
                t = tab
            )?;
            let empty = vec![];
            for &src in nodes {
                let edges = self.edges.get(src).unwrap_or(&empty);
                for edge in edges {
                    if !self.handoffs.contains_key(src) && !self.handoffs.contains_key(edge.dst) {
                        write_dot_edge(src, edge.dst, tab, w)?;
                    }
                }
            }
            tab -= 2;
            writeln!(w, "{:t$}}}", "", t = tab)?;
            tab -= 2;
            writeln!(w, "{:t$}}}", "", t = tab)?;
        }

        //write out edges adjacent to handoffs outside the clusters
        for (src, edges) in self.edges.iter() {
            for edge in edges {
                if self.handoffs.contains_key(src) || self.handoffs.contains_key(edge.dst) {
                    write_dot_edge(src, edge.dst, tab, w)?;
                }
            }
        }

        tab -= 2;
        writeln!(w, "{:t$}}}", "", t = tab)?;
        Ok(())
    }
}
