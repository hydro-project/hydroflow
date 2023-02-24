use std::collections::BTreeMap;

use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};

use serde::{Deserialize, Serialize};

use super::{Color, GraphNodeId, GraphSubgraphId};
use crate::graph::ops::{FlowProperties, FlowPropertyVal};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SerdeEdge {
    pub src: GraphNodeId,
    pub dst: GraphNodeId,
    pub blocking: bool,
    pub label: Option<String>,
    pub properties: FlowProperties,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SerdeGraph {
    pub nodes: SecondaryMap<GraphNodeId, String>,
    pub node_color_map: SparseSecondaryMap<GraphNodeId, Color>,
    pub edges: SecondaryMap<GraphNodeId, Vec<SerdeEdge>>,
    pub barrier_handoffs: SparseSecondaryMap<GraphNodeId, bool>,
    pub subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    pub subgraph_stratum: SecondaryMap<GraphSubgraphId, usize>,
    pub subgraph_internal_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
    pub node_properties: SecondaryMap<GraphNodeId, FlowProperties>,
    /// What variable name each graph node belongs to (if any).
    /// The nodes that each variable name encompases.
    pub varname_nodes: BTreeMap<String, Vec<GraphNodeId>>,
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
        let mut tab: usize = 0;

        fn write_mermaid_prelude(write: &mut impl std::fmt::Write) -> std::fmt::Result {
            // intro
            writeln!(
                write,
                r"%%{{init:{{'theme':'base','themeVariables':{{'clusterBkg':'#ddd','clusterBorder':'#888'}}}}}}%%",
            )?;
            writeln!(write, "flowchart TD")?;
            writeln!(write, "classDef pullClass fill:#02f,color:#fff,stroke:#000")?;
            writeln!(write, "classDef pushClass fill:#ff0,stroke:#000")?;

            writeln!(
                write,
                "linkStyle default stroke:#aaa,stroke-width:4px,color:red,font-size:1.5em;"
            )?;
            Ok(())
        }
        fn write_mermaid_node(
            node_id: GraphNodeId,
            node_color_map: &SparseSecondaryMap<GraphNodeId, Color>,
            text: &str,
            tab: usize,
            write: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            let mode = match node_color_map.get(node_id) {
                Some(color) => *color,
                None => Color::Comp,
            };

            let class_str = match mode {
                Color::Push => "pushClass",
                Color::Pull => "pullClass",
                _ => "otherClass",
            }
            .to_string();
            let label = format!(
                r#"{:t$}{id:?}{lbracket}"({id:?}) <tt>{code}</tt>"{rbracket}:::{class}"#,
                "",
                id = node_id.data(),
                class = class_str,
                lbracket = match mode {
                    Color::Push => r"[/",
                    Color::Pull => r"[\",
                    _ => "[",
                },
                code = text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\n', "<br>"),
                rbracket = match mode {
                    Color::Push => r"\]",
                    Color::Pull => r"/]",
                    _ => "]",
                },
                t = tab,
            );
            writeln!(write, "{}", label)?;
            Ok(())
        }

        fn write_mermaid_edge(
            src: GraphNodeId,
            edge: &SerdeEdge,
            tab: usize,
            write: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            let src_str = format!("{:?}", src.data());
            let dest_str = format!("{:?}", edge.dst.data());
            writeln!(
                write,
                "{:t$}{}{}{}{}",
                "",
                src_str.trim(),
                if let Some(label) = &edge.label {
                    if edge.blocking {
                        format!("=={}", label.trim())
                    } else {
                        format!("--{}", label.trim())
                    }
                } else {
                    "".to_string()
                },
                if edge.blocking { "===o" } else { "--->" },
                dest_str.trim(),
                t = tab,
            )?;
            Ok(())
        }

        fn write_mermaid_subgraph_start(
            subgraph_id: GraphSubgraphId,
            stratum: usize,
            tab: usize,
            write: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            writeln!(
                write,
                "{:t$}subgraph sg_{sg:?} [\"sg_{sg:?} stratum {:?}\"]",
                "",
                stratum,
                sg = subgraph_id.data(),
                t = tab,
            )?;
            Ok(())
        }

        fn write_mermaid_subgraph_end(
            write: &mut impl std::fmt::Write,
            tab: usize,
        ) -> std::fmt::Result {
            // subgraph footer
            writeln!(write, "{:t$}end", "", t = tab)?;
            Ok(())
        }

        fn write_mermaid_subgraph_varnames<'a>(
            write: &mut impl std::fmt::Write,
            subgraph_id: GraphSubgraphId,
            varname: &str,
            local_named_nodes: impl Iterator<Item = &'a GraphNodeId>,
            tab: usize,
        ) -> std::fmt::Result {
            writeln!(
                write,
                "{:t$}subgraph sg_{sg:?}_var_{var} [\"var <tt>{var}</tt>\"]",
                "",
                sg = subgraph_id.data(),
                var = varname,
                t = tab,
            )?;
            for local_named_node in local_named_nodes {
                writeln!(write, "{:t$}{:?}", "", local_named_node.data(), t = tab + 4)?;
            }
            writeln!(write, "{:t$}end", "", t = tab)?;
            Ok(())
        }

        write_mermaid_prelude(write)?;
        for (subgraph_id, node_ids) in self.subgraph_nodes.iter() {
            let stratum = self.subgraph_stratum.get(subgraph_id);
            write_mermaid_subgraph_start(subgraph_id, *stratum.unwrap(), tab, write)?;
            tab += 4;

            // write out nodes
            for &node_id in node_ids.iter() {
                write_mermaid_node(
                    node_id,
                    &self.node_color_map,
                    self.nodes.get(node_id).unwrap(),
                    tab,
                    write,
                )?;
            }
            // write out internal handoffs
            let empty = vec![];
            if let Some(hoffs) = self.subgraph_internal_handoffs.get(subgraph_id) {
                for hoff in hoffs.iter() {
                    write_mermaid_node(
                        *hoff,
                        &self.node_color_map,
                        self.nodes.get(*hoff).unwrap(),
                        tab,
                        write,
                    )?;
                    // write out internal handoff edges
                    for edge in self.edges.get(*hoff).unwrap_or(&empty) {
                        write_mermaid_edge(*hoff, edge, tab, write)?;
                    }
                }
            }

            // write out edges
            for &src in node_ids.iter() {
                if let Some(edges) = self.edges.get(src) {
                    for edge in edges {
                        if !self.barrier_handoffs.contains_key(edge.dst) {
                            write_mermaid_edge(src, edge, tab, write)?;
                        }
                    }
                }
            }

            // write out any variable names
            for (varname, varname_node_ids) in self.varname_nodes.iter() {
                // TODO(mingwei): this is awkward, inefficient runtime
                let mut local_named_nodes = varname_node_ids
                    .iter()
                    .filter(|node_id| node_ids.contains(node_id))
                    .peekable();
                if local_named_nodes.peek().is_some() {
                    write_mermaid_subgraph_varnames(
                        write,
                        subgraph_id,
                        varname,
                        local_named_nodes,
                        tab,
                    )?;
                }
            }

            tab -= 4;
            write_mermaid_subgraph_end(write, tab)?;
        }

        //write out handoffs outside the clusters and adjacent edges
        for (src, edges) in self.edges.iter() {
            for edge in edges {
                if self.barrier_handoffs.contains_key(src) {
                    // write out handoff
                    write_mermaid_node(
                        src,
                        &self.node_color_map,
                        self.nodes.get(src).unwrap(),
                        tab,
                        write,
                    )?;
                    // write out edge
                    write_mermaid_edge(src, edge, tab, write)?;
                } else if self.barrier_handoffs.contains_key(edge.dst) {
                    // just write out edge
                    write_mermaid_edge(src, edge, tab, write)?;
                }
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
        // intro
        let mut tab: usize = 4;

        fn write_dot_prelude(write: &mut impl std::fmt::Write) -> std::fmt::Result {
            writeln!(write, "digraph {{")?;
            Ok(())
        }
        fn write_dot_postlude(tab: usize, write: &mut impl std::fmt::Write) -> std::fmt::Result {
            writeln!(write, "{:t$}}}", "", t = tab)?;
            Ok(())
        }

        fn write_dot_node(
            node_id: GraphNodeId,
            node_color_map: &SparseSecondaryMap<GraphNodeId, Color>,
            node_properties: &SecondaryMap<GraphNodeId, FlowProperties>,
            text: &str,
            tab: usize,
            w: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            let mut properties = Vec::new();
            let nm = text.replace('"', "\\\"").replace('\n', "\\l");
            let label = format!("n{:?}", node_id.data());
            properties.push(format!(
                "label=\"({}) {}{}\"",
                label,
                nm,
                // if contains linebreak left-justify by appending another "\\l"
                if nm.contains("\\l") { "\\l" } else { "" },
            ));

            properties.push("fontname=Monaco".to_string());

            let mode = match node_color_map.get(node_id) {
                Some(color) => *color,
                None => Color::Hoff,
            };
            let shape_str = match mode {
                // Color::Push => "polygon, sides=4, distortion=-0.1",
                // Color::Pull => "polygon, sides=4, distortion=0.1",
                Color::Push => "house",
                Color::Pull => "invhouse",
                Color::Hoff => "parallelogram",
                Color::Comp => "circle",
            };
            properties.push(format!("shape = {}", shape_str));

            let dashed_str = if node_properties[node_id].deterministic == FlowPropertyVal::No {
                ",dashed"
            } else {
                ""
            };
            let color_str = if node_properties[node_id].tainted {
                "color = \"#ff0000\""
            } else {
                match mode {
                    Color::Push => "color = \"#ffff00\"",
                    Color::Pull => "color = \"#0022ff\", fontcolor = \"#ffffff\"",
                    Color::Hoff => "color = \"#ddddff\"",
                    Color::Comp => "color = white",
                }
            };
            properties.push(format!("style=\"filled{}\", {}", dashed_str, color_str));
            writeln!(
                w,
                "{:t$}n{:?} {}",
                "",
                node_id.data(),
                if !properties.is_empty() {
                    format!(" [{}]", properties.join(", "))
                } else {
                    "".to_string()
                },
                t = tab,
            )?;
            Ok(())
        }

        fn write_dot_edge(
            src: GraphNodeId,
            edge: &SerdeEdge,
            tab: usize,
            w: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            let mut properties = Vec::new();
            if edge.label.is_some() {
                properties.push(format!("label=\"{}\"", edge.label.as_ref().unwrap()));
            };
            // if edge.blocking {
            //     properties.push("arrowhead=box, color=red".to_string());
            // };
            if edge.properties.tainted {
                properties.push("color=red".to_string());
            };
            if edge.properties.deterministic == FlowPropertyVal::No {
                properties.push("style=dashed".to_string());
            }
            writeln!(
                w,
                "{:t$}n{:?} -> n{:?}{}",
                "",
                src.data(),
                edge.dst.data(),
                if !properties.is_empty() {
                    format!(" [{}]", properties.join(", "))
                } else {
                    "".to_string()
                },
                t = tab,
            )?;
            Ok(())
        }

        fn write_dot_subgraph_start(
            subgraph_id: GraphSubgraphId,
            stratum: usize,
            tab: usize,
            write: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            writeln!(
                write,
                "{:t$}subgraph \"cluster n{:?}\" {{",
                "",
                subgraph_id.data(),
                t = tab
            )?;
            writeln!(write, "{:t$}fillcolor=\"#dddddd\"", "", t = tab + 4)?;
            writeln!(write, "{:t$}style=filled", "", t = tab + 4)?;
            writeln!(
                write,
                "{:t$}label = \"sg_{:?}\\nstratum {}\"",
                "",
                subgraph_id.data(),
                stratum,
                t = tab + 4
            )?;
            Ok(())
        }

        fn write_dot_subgraph_end(
            write: &mut impl std::fmt::Write,
            tab: usize,
        ) -> std::fmt::Result {
            // subgraph footer
            writeln!(write, "{:t$}}}", "", t = tab)?;
            Ok(())
        }

        write_dot_prelude(w)?;
        for (subgraph_id, node_ids) in self.subgraph_nodes.iter() {
            let stratum = self.subgraph_stratum.get(subgraph_id);
            // subgraph header
            write_dot_subgraph_start(subgraph_id, *stratum.unwrap(), tab, w)?; // TODO: unwrap
            tab += 4;

            // write out nodes
            for &node_id in node_ids.iter() {
                // write out node
                write_dot_node(
                    node_id,
                    &self.node_color_map,
                    &self.node_properties,
                    self.nodes.get(node_id).unwrap(),
                    tab,
                    w,
                )?;
            }
            // write out internal handoffs
            let empty = vec![];
            if let Some(hoffs) = self.subgraph_internal_handoffs.get(subgraph_id) {
                for hoff in hoffs {
                    let text = self.nodes.get(*hoff).unwrap();
                    write_dot_node(
                        *hoff,
                        &self.node_color_map,
                        &self.node_properties,
                        text,
                        tab,
                        w,
                    )?;
                    // write out internal handoff edges
                    for edge in self.edges.get(*hoff).unwrap_or(&empty) {
                        write_dot_edge(*hoff, edge, tab, w)?;
                    }
                }
            }

            // write out edges
            for &src in node_ids.iter() {
                if let Some(edges) = self.edges.get(src) {
                    for edge in edges {
                        if !self.barrier_handoffs.contains_key(edge.dst) {
                            // write out edge
                            write_dot_edge(src, edge, tab, w)?;
                        }
                    }
                }
            }
            // subgraph footer
            tab -= 4;
            write_dot_subgraph_end(w, tab)?;
        }
        //write out handoffs outside the clusters and adjacent edges
        for (src, edges) in self.edges.iter() {
            for edge in edges {
                if self.barrier_handoffs.contains_key(src) {
                    // write out handoff
                    let text = self.nodes.get(src).unwrap();
                    write_dot_node(
                        src,
                        &self.node_color_map,
                        &self.node_properties,
                        text,
                        tab,
                        w,
                    )?;
                    // write out edge
                    write_dot_edge(src, edge, tab, w)?;
                } else if self.barrier_handoffs.contains_key(edge.dst) {
                    // just write out edge
                    write_dot_edge(src, edge, tab, w)?;
                }
            }
        }

        // outro
        write_dot_postlude(tab, w)?;
        Ok(())
    }
}
