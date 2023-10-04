#![cfg(feature = "debugging")]

use std::fmt::Write;
use std::io::Result;

use super::HydroflowGraph;

impl HydroflowGraph {
    /// Opens this as a mermaid graph in the [mermaid.live](https://mermaid.live) browser editor.
    pub fn open_mermaid(&self) -> Result<()> {
        let mermaid_src = self.to_mermaid();
        let state = serde_json::json!({
            "code": mermaid_src,
            "mermaid": serde_json::json!({
                "theme": "default"
            }),
            "autoSync": true,
            "updateDiagram": true
        });
        let state_json = serde_json::to_vec(&state)?;
        let state_base64 = data_encoding::BASE64URL.encode(&state_json);
        webbrowser::open(&format!(
            "https://mermaid.live/edit#base64:{}",
            state_base64
        ))
    }

    /// Opens this as dot/graphviz graph in the [Graphviz Online](https://dreampuf.github.io/GraphvizOnline/#)
    /// browser editor.
    pub fn open_dot(&self) -> Result<()> {
        let dot_src = self.to_dot();
        let mut url = "https://dreampuf.github.io/GraphvizOnline/#".to_owned();
        for byte in dot_src.bytes() {
            // Lazy percent encoding: https://en.wikipedia.org/wiki/Percent-encoding
            write!(url, "%{:02x}", byte).unwrap();
        }
        webbrowser::open(&url)
    }
}
