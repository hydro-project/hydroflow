use std::collections::HashMap;
use std::error::Error;
use std::iter::repeat;

use crate::cli_refs::ServerBindConfig;
use crate::config::{CustomPort, MaelstromPort, Port};
use crate::hydro_interact::{DemuxSink, DirectSink, SinkConnection};

impl Port {
    /// Returns the (port_name, serverbind config) for sink ports
    pub fn bind_config(&self, node_count: usize) -> Option<(&str, ServerBindConfig)> {
        match self {
            Self::Sink(_) => None,
            Self::Source(port) => Some(port.bind_config()),
            Self::Custom(port) => Some(port.bind_config(node_count)),
        }
    }

    /// Binds to sink ports
    pub async fn bind(&self, node_count: usize) -> Result<Option<SinkConnection>, Box<dyn Error>> {
        match self {
            Self::Source(_) => Ok(None),
            Self::Sink(port) => port.bind().await.map(Some),
            Self::Custom(port) => port.bind(node_count).await.map(Some),
        }
    }

    pub fn maelstrom_type(&self) -> &str {
        match self {
            Self::Sink(port) => port.maelstrom_type(),
            Self::Source(port) => port.maelstrom_type(),
            Self::Custom(port) => port.maelstrom_type(),
        }
    }

    pub fn source_port_name(&self) -> Option<&str> {
        match self {
            Self::Sink(_) => None,
            Self::Source(port) => Some(port.port_name()),
            Self::Custom(port) => Some(port.tagged_port_name()),
        }
    }
}

impl MaelstromPort {
    /// Returns (port name, tcp port)
    fn bind_config(&self) -> (&str, ServerBindConfig) {
        let port_name = self.port_name();
        let localhost = ServerBindConfig::TcpPort("127.0.0.1".to_string());
        (port_name, localhost)
    }

    async fn bind(&self) -> Result<SinkConnection, Box<dyn Error>> {
        let connection = DirectSink::bind(self).await?;
        Ok(SinkConnection::Direct(connection))
    }
}

impl CustomPort {
    /// Returns (sink port name, merge of all possible tagged inputs)
    fn bind_config(&self, node_count: usize) -> (&str, ServerBindConfig) {
        let port_name = self.tagged_port_name();

        let localhost = ServerBindConfig::TcpPort("127.0.0.1".to_string());

        let tagged_configs = repeat(localhost)
            .enumerate()
            .map(|(i, tcpport)| (i as u32, Box::new(tcpport)))
            .map(|(i, tcpport)| ServerBindConfig::Tagged(tcpport, i))
            .take(node_count)
            .collect();

        (port_name, ServerBindConfig::Merge(tagged_configs))
    }

    async fn bind(&self, node_count: usize) -> Result<SinkConnection, Box<dyn Error>> {
        let connection = DemuxSink::bind(self, node_count).await?;
        Ok(SinkConnection::Demux(connection))
    }
}

/// Returns the setup string which sets up all source ports.
pub fn make_setup_string<'a, I: IntoIterator<Item = &'a Port>>(
    ports: I,
    node_count: usize,
) -> Result<String, Box<dyn Error>> {
    // Pairs from port name to bind configuration for all source ports
    let source_setup_pairs = ports
        .into_iter()
        .flat_map(|p| p.bind_config(node_count))
        .collect::<HashMap<_, _>>();

    let source_setup_string = serde_json::to_string(&source_setup_pairs)?;

    Ok(source_setup_string)
}

pub fn source_name_to_type<'a, I: IntoIterator<Item = &'a Port>>(
    ports: I,
) -> HashMap<String, String> {
    ports
        .into_iter()
        .filter_map(|p| {
            p.source_port_name()
                .map(|port_name| (port_name, p.maelstrom_type()))
        })
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

pub async fn bind_to_sinks<'a, I: IntoIterator<Item = &'a Port>>(
    ports: I,
    node_count: usize,
) -> Result<Vec<SinkConnection>, Box<dyn Error>> {
    let mut connections = Vec::new();
    for port in ports {
        let binding = port.bind(node_count).await?;
        if let Some(binding) = binding {
            connections.push(binding);
        }
    }

    Ok(connections)
}

pub fn make_start_string(
    sink_connections: &[SinkConnection],
    node_id: usize,
    node_names: &[String],
) -> Result<String, Box<dyn Error>> {
    let connection_defns = sink_connections
        .iter()
        .map(|connection| connection.defn())
        .collect::<HashMap<_, _>>();

    let node_names: HashMap<usize, &str> =
        node_names.iter().map(|x| x.as_str()).enumerate().collect();
    let data = (connection_defns, node_id, node_names);

    let raw_string = serde_json::to_string(&data)?;
    let formatted_string = format!("start: {raw_string}");

    Ok(formatted_string)
}
