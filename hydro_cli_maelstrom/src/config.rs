use std::path::PathBuf;

/// A configuration for hydro deploy maelstrom wrapper which identifies how to map ports and maelstrom payloads.
pub struct Config {
    pub binary: PathBuf,
    pub ports: Vec<Port>,
}

/// A logical port
pub enum Port {
    Source(MaelstromPort),
    Sink(MaelstromPort),
    Custom(CustomPort),
}

/// A port which communicates through wrapped maelstrom payloads.
pub struct MaelstromPort {
    port_name: String,
    maelstrom_type: String,
}

/// A custom two-sided port with a demux out and tagged in.
/// Used for passing around non-maelstrom payloads between nodes.
pub struct CustomPort {
    tagged_port_name: String,
    demux_port_name: String,
    maelstrom_type: String,
}

impl MaelstromPort {
    pub fn new(port_name: String, maelstrom_type: String) -> MaelstromPort {
        MaelstromPort {
            port_name,
            maelstrom_type,
        }
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    pub fn maelstrom_type(&self) -> &str {
        &self.maelstrom_type
    }
}

const CUSTOM_TAG: &str = "_~*hydromael*~_";

impl CustomPort {
    pub fn new(tagged_port_name: String, demux_port_name: String) -> CustomPort {
        // Initialize maelstrom type for custom port to a unique name which doesn't conflict with maelstrom provided types
        let maelstrom_type = format!(
            "{}{}{}{}",
            CUSTOM_TAG, demux_port_name, CUSTOM_TAG, tagged_port_name
        );
        CustomPort {
            tagged_port_name,
            demux_port_name,
            maelstrom_type,
        }
    }

    pub fn tagged_port_name(&self) -> &str {
        &self.tagged_port_name
    }

    pub fn demux_port_name(&self) -> &str {
        &self.demux_port_name
    }

    pub fn maelstrom_type(&self) -> &str {
        &self.maelstrom_type
    }
}
