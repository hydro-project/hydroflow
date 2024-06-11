use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

/// L0 Data Store settings.
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerSettings {
    /// An initial set of "seed nodes" that can be used to bootstrap the gossip cluster. These
    /// won't be the only nodes in the cluster, but they can be used to discover other nodes.
    pub seed_nodes: Vec<SeedNodeSettings>,
}

impl ServerSettings {
    /// Load the settings from the configuration files.
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let settings = Config::builder()
            /* Load the default settings from the `config/default.toml` file. */
            .add_source(File::with_name("config/default"))

            /* Load additional overrides based on context (alpha, beta, production, etc.), if they exist. */
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))

            /* Load the local settings, if they exist. These are .gitignored to prevent accidental
               check-in. */
            .add_source(File::with_name("config/local").required(false))
            .build()?;

        settings.try_deserialize()
    }
}

/// A seed node that can be used to bootstrap the gossip cluster.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct SeedNodeSettings {
    /// The ID of the seed node.
    pub id: String,

    /// The IP address on which the seed node is listening for gossip messages.
    pub ip: String,

    /// The port on which the seed node is listening for gossip messages.
    pub port: u16,
}
