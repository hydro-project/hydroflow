use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use async_process::{Command, Stdio};
use async_trait::async_trait;
use hydroflow_cli_integration::ServerBindConfig;
use tokio::sync::RwLock;

use super::{
    ClientStrategy, Host, HostTargetType, LaunchedBinary, LaunchedHost, ResourceBatch,
    ResourceResult, ServerStrategy,
};
use crate::hydroflow_crate::BuiltCrate;

pub mod launched_binary;
pub use launched_binary::*;

#[derive(Debug)]
pub struct LocalhostHost {
    pub id: usize,
    client_only: bool,
}

impl LocalhostHost {
    pub fn new(id: usize) -> LocalhostHost {
        LocalhostHost {
            id,
            client_only: false,
        }
    }

    pub fn client_only(&self) -> LocalhostHost {
        LocalhostHost {
            id: self.id,
            client_only: true,
        }
    }
}

#[async_trait]
impl Host for LocalhostHost {
    fn target_type(&self) -> HostTargetType {
        HostTargetType::Local
    }

    fn request_port(&mut self, _bind_type: &ServerStrategy) {}
    fn collect_resources(&self, _resource_batch: &mut ResourceBatch) {}
    fn request_custom_binary(&mut self) {}

    fn id(&self) -> usize {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn launched(&self) -> Option<Arc<dyn LaunchedHost>> {
        Some(Arc::new(LaunchedLocalhost {}))
    }

    async fn provision(&mut self, _resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {})
    }

    fn strategy_as_server<'a>(
        &'a self,
        connection_from: &dyn Host,
    ) -> Result<(
        ClientStrategy<'a>,
        Box<dyn FnOnce(&mut dyn std::any::Any) -> ServerStrategy>,
    )> {
        if self.client_only {
            anyhow::bail!("Localhost cannot be a server if it is client only")
        }

        if connection_from.can_connect_to(ClientStrategy::UnixSocket(self.id)) {
            Ok((
                ClientStrategy::UnixSocket(self.id),
                Box::new(|_| ServerStrategy::UnixSocket),
            ))
        } else if connection_from.can_connect_to(ClientStrategy::InternalTcpPort(self)) {
            Ok((
                ClientStrategy::InternalTcpPort(self),
                Box::new(|_| ServerStrategy::InternalTcpPort),
            ))
        } else {
            anyhow::bail!("Could not find a strategy to connect to localhost")
        }
    }

    fn can_connect_to(&self, typ: ClientStrategy) -> bool {
        match typ {
            ClientStrategy::UnixSocket(id) => {
                #[cfg(unix)]
                {
                    self.id == id
                }

                #[cfg(not(unix))]
                {
                    let _ = id;
                    false
                }
            }
            ClientStrategy::InternalTcpPort(target_host) => self.id == target_host.id(),
            ClientStrategy::ForwardedTcpPort(_) => true,
        }
    }
}

struct LaunchedLocalhost {}

#[async_trait]
impl LaunchedHost for LaunchedLocalhost {
    fn server_config(&self, bind_type: &ServerStrategy) -> ServerBindConfig {
        match bind_type {
            ServerStrategy::UnixSocket => ServerBindConfig::UnixSocket,
            ServerStrategy::InternalTcpPort => ServerBindConfig::TcpPort("127.0.0.1".to_string()),
            ServerStrategy::ExternalTcpPort(_) => panic!("Cannot bind to external port"),
            ServerStrategy::Demux(demux) => {
                let mut config_map = HashMap::new();
                for (key, underlying) in demux {
                    config_map.insert(*key, self.server_config(underlying));
                }

                ServerBindConfig::Demux(config_map)
            }
            ServerStrategy::Merge(merge) => {
                let mut configs = vec![];
                for underlying in merge {
                    configs.push(self.server_config(underlying));
                }

                ServerBindConfig::Merge(configs)
            }
            ServerStrategy::Tagged(underlying, id) => {
                ServerBindConfig::Tagged(Box::new(self.server_config(underlying)), *id)
            }
            ServerStrategy::Null => ServerBindConfig::Null,
        }
    }

    async fn copy_binary(&self, _binary: Arc<BuiltCrate>) -> Result<()> {
        Ok(())
    }

    async fn launch_binary(
        &self,
        id: String,
        binary: Arc<BuiltCrate>,
        args: &[String],
        perf: Option<PathBuf>,
    ) -> Result<Arc<RwLock<dyn LaunchedBinary>>> {
        let mut command = if let Some(perf) = perf {
            println!("Profiling binary with perf");
            let mut tmp = Command::new("perf");
            tmp.args(["record", "-F", "5", "--call-graph", "dwarf,64000", "-o"])
                .arg(&perf)
                .arg(&binary.bin_path)
                .args(args);
            tmp
        } else {
            let mut tmp = Command::new(&binary.bin_path);
            tmp.args(args);
            tmp
        };

        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(not(unix))]
        command.kill_on_drop(true);

        let child = command
            .spawn()
            .with_context(|| format!("Failed to execute command: {:?}", command))?;

        Ok(Arc::new(RwLock::new(LaunchedLocalhostBinary::new(
            child, id,
        ))))
    }

    async fn forward_port(&self, addr: &SocketAddr) -> Result<SocketAddr> {
        Ok(*addr)
    }
}
