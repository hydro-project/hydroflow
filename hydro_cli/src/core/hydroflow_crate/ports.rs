use super::HydroflowCrate;
use crate::core::{ClientStrategy, Host, LaunchedHost, ServerStrategy};

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use dyn_clone::DynClone;

use std::collections::HashMap;

use std::sync::{Arc, Weak};
use tokio::sync::RwLock;

use hydroflow_cli_integration::ServerPort;

pub trait HydroflowSource: Send + Sync {
    fn send_to(&mut self, to: &mut dyn HydroflowSink);
}

#[async_trait]
pub trait HydroflowServer: DynClone + Send + Sync {
    async fn get_port(&self) -> ServerPort;
    async fn launched_host(&self) -> Arc<dyn LaunchedHost>;
}

pub trait HydroflowSink: Send + Sync {
    /// Instantiate the sink as the source host connecting to the sink host.
    fn instantiate(&self, client_host: &Arc<RwLock<dyn Host>>) -> Result<ServerConfig>;

    /// Instantiate the sink, but as the sink host connecting to the source host.
    fn instantiate_reverse(
        &mut self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ServerStrategy>;
}

pub struct DemuxSink {
    pub demux: HashMap<u32, Arc<RwLock<dyn HydroflowSink>>>,
}

impl HydroflowSink for DemuxSink {
    fn instantiate(&self, client_host: &Arc<RwLock<dyn Host>>) -> Result<ServerConfig> {
        let mut instantiated_map = HashMap::new();
        for (key, target) in &self.demux {
            instantiated_map.insert(*key, target.try_read().unwrap().instantiate(client_host)?);
        }

        Ok(ServerConfig::Demux(instantiated_map))
    }

    fn instantiate_reverse(
        &mut self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ServerStrategy> {
        let mut instantiated_map = HashMap::new();
        for (key, target) in &self.demux {
            instantiated_map.insert(
                *key,
                target.try_write().unwrap().instantiate_reverse(
                    server_host,
                    dyn_clone::clone_box(&*server_sink),
                    // the parent wrapper selects the demux port for the parent defn, so do that first
                    &|p| ServerConfig::DemuxSelect(Box::new(wrap_client_port(p)), *key),
                )?,
            );
        }

        Ok(ServerStrategy::Demux(instantiated_map))
    }
}

#[derive(Clone)]
pub struct HydroflowPortConfig {
    pub service: Weak<RwLock<HydroflowCrate>>,
    pub port: String,
}

impl HydroflowSource for HydroflowPortConfig {
    fn send_to(&mut self, sink: &mut dyn HydroflowSink) {
        let from = self.service.upgrade().unwrap();
        let mut from_write = from.try_write().unwrap();
        from_write
            .add_connection(&from, self.port.clone(), sink)
            .unwrap();
    }
}

#[async_trait]
impl HydroflowServer for HydroflowPortConfig {
    async fn get_port(&self) -> ServerPort {
        let server = self.service.upgrade().unwrap();
        let server = server.read().await;
        server.server_defns.get(&self.port).unwrap().clone()
    }

    async fn launched_host(&self) -> Arc<dyn LaunchedHost> {
        let server = self.service.upgrade().unwrap();
        let server_read = server.read().await;
        server_read.launched_host.as_ref().unwrap().clone()
    }
}

impl HydroflowSink for HydroflowPortConfig {
    fn instantiate(&self, client_host: &Arc<RwLock<dyn Host>>) -> Result<ServerConfig> {
        let server = self.service.upgrade().unwrap();
        let mut server_write = server.try_write().unwrap();

        let client_host_id = client_host.try_read().unwrap().id();

        let server_host_clone = server_write.on.clone();
        let mut server_host = server_host_clone.try_write().unwrap();

        let client_host_read = if server_host.id() == client_host_id {
            None
        } else {
            client_host.try_read().ok()
        };

        let (conn_type, bind_type) = server_host.strategy_as_server(client_host_read.as_deref())?;

        server_write
            .port_to_bind
            .insert(self.port.clone(), bind_type);

        Ok(ServerConfig::from_strategy(
            &conn_type,
            Box::new(self.clone()),
        ))
    }

    fn instantiate_reverse(
        &mut self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ServerStrategy> {
        let client = self.service.upgrade().unwrap();
        let mut client_write = client.try_write().unwrap();

        let client_host_id = client_write.on.try_read().unwrap().id();

        let server_host_clone = server_host.clone();
        let mut server_host = server_host_clone.try_write().unwrap();

        let client_host_clone = client_write.on.clone();
        let client_host_read = if server_host.id() == client_host_id {
            None
        } else {
            client_host_clone.try_read().ok()
        };

        let (conn_type, bind_type) = server_host.strategy_as_server(client_host_read.as_deref())?;

        client_write.port_to_server.insert(
            self.port.clone(),
            wrap_client_port(ServerConfig::from_strategy(&conn_type, server_sink)),
        );

        Ok(bind_type)
    }
}

pub enum ServerConfig {
    Direct(Box<dyn HydroflowServer>),
    Forwarded(Box<dyn HydroflowServer>),
    /// A demux that will be used at runtime.
    Demux(HashMap<u32, ServerConfig>),
    /// The other side of a demux, with a port to extract the appropriate connection.
    DemuxSelect(Box<ServerConfig>, u32),
}

impl ServerConfig {
    pub fn from_strategy(
        strategy: &ClientStrategy,
        server: Box<dyn HydroflowServer>,
    ) -> ServerConfig {
        match strategy {
            ClientStrategy::UnixSocket(_) | ClientStrategy::InternalTcpPort(_) => {
                ServerConfig::Direct(server)
            }
            ClientStrategy::ForwardedTcpPort(_) => ServerConfig::Forwarded(server),
        }
    }
}

#[async_recursion]
async fn forward_connection(conn: &ServerPort, target: &dyn LaunchedHost) -> ServerPort {
    match conn {
        ServerPort::UnixSocket(_) => panic!("Expected a TCP port to be forwarded"),
        ServerPort::TcpPort(addr) => ServerPort::TcpPort(target.forward_port(addr).await.unwrap()),
        ServerPort::Demux(demux) => {
            let mut forwarded_map = HashMap::new();
            for (key, conn) in demux {
                forwarded_map.insert(*key, forward_connection(conn, target).await);
            }
            ServerPort::Demux(forwarded_map)
        }
    }
}

impl ServerConfig {
    #[async_recursion]
    pub async fn sink_port(&self) -> ServerPort {
        match self {
            ServerConfig::Direct(server) => server.get_port().await,

            ServerConfig::Forwarded(server) => {
                forward_connection(
                    &server.get_port().await,
                    server.launched_host().await.as_ref(),
                )
                .await
            }

            ServerConfig::Demux(demux) => {
                let mut demux_map = HashMap::new();
                for (key, conn) in demux {
                    demux_map.insert(*key, conn.sink_port().await);
                }
                ServerPort::Demux(demux_map)
            }

            ServerConfig::DemuxSelect(underlying, key) => {
                if let ServerPort::Demux(mut mapping) = underlying.sink_port().await {
                    mapping.remove(key).unwrap()
                } else {
                    panic!("Expected a demux connection definition")
                }
            }
        }
    }
}
