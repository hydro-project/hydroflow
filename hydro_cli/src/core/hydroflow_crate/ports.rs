use super::HydroflowCrate;
use crate::core::{ClientStrategy, Host, HostStrategyGetter, LaunchedHost, ServerStrategy};

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use dyn_clone::DynClone;

use std::any::Any;
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

pub type ReverseSinkInstantiator = Box<dyn FnOnce(&mut dyn Any) -> ServerStrategy>;

pub trait HydroflowSink: Send + Sync {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Instantiate the sink as the source host connecting to the sink host.
    /// Returns a thunk that can be called to perform mutations that instantiate the sink.
    fn instantiate(&self, client_path: &SourcePath) -> Result<Box<dyn FnOnce() -> ServerConfig>>;

    /// Instantiate the sink, but as the sink host connecting to the source host.
    /// Returns a thunk that can be called to perform mutations that instantiate the sink, taking a mutable reference to this sink.
    fn instantiate_reverse(
        &self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ReverseSinkInstantiator>;
}

pub struct NullSourceSink;

impl HydroflowSource for NullSourceSink {
    fn send_to(&mut self, to: &mut dyn HydroflowSink) {
        to.instantiate(&SourcePath::Null).unwrap()();
    }
}

impl HydroflowSink for NullSourceSink {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn instantiate(&self, _client_path: &SourcePath) -> Result<Box<dyn FnOnce() -> ServerConfig>> {
        Ok(Box::new(|| ServerConfig::Null))
    }

    fn instantiate_reverse(
        &self,
        _server_host: &Arc<RwLock<dyn Host>>,
        _server_sink: Box<dyn HydroflowServer>,
        _wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ReverseSinkInstantiator> {
        Ok(Box::new(|_| ServerStrategy::Null))
    }
}

pub struct DemuxSink {
    pub demux: HashMap<u32, Arc<RwLock<dyn HydroflowSink>>>,
}

impl HydroflowSink for DemuxSink {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn instantiate(&self, client_host: &SourcePath) -> Result<Box<dyn FnOnce() -> ServerConfig>> {
        let mut thunk_map = HashMap::new();
        for (key, target) in &self.demux {
            thunk_map.insert(*key, target.try_read().unwrap().instantiate(client_host)?);
        }

        Ok(Box::new(move || {
            let instantiated_map = thunk_map
                .into_iter()
                .map(|(key, thunk)| (key, thunk()))
                .collect();

            ServerConfig::Demux(instantiated_map)
        }))
    }

    fn instantiate_reverse(
        &self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<Box<dyn FnOnce(&mut dyn Any) -> ServerStrategy>> {
        let mut thunk_map = HashMap::new();
        for (key, target) in &self.demux {
            thunk_map.insert(
                *key,
                target.try_write().unwrap().instantiate_reverse(
                    server_host,
                    dyn_clone::clone_box(&*server_sink),
                    // the parent wrapper selects the demux port for the parent defn, so do that first
                    &|p| ServerConfig::DemuxSelect(Box::new(wrap_client_port(p)), *key),
                )?,
            );
        }

        Ok(Box::new(move |me| {
            let me = me.downcast_mut::<DemuxSink>().unwrap();
            let instantiated_map = thunk_map
                .into_iter()
                .map(|(key, thunk)| {
                    (
                        key,
                        thunk(
                            me.demux
                                .get_mut(&key)
                                .unwrap()
                                .try_write()
                                .unwrap()
                                .as_any_mut(),
                        ),
                    )
                })
                .collect();

            ServerStrategy::Demux(instantiated_map)
        }))
    }
}

#[derive(Clone)]
pub struct HydroflowPortConfig {
    pub service: Weak<RwLock<HydroflowCrate>>,
    pub port: String,
    pub merge: bool,
}

impl HydroflowPortConfig {
    pub fn merge(&self) -> Self {
        Self {
            service: self.service.clone(),
            port: self.port.clone(),
            merge: true,
        }
    }
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

pub enum SourcePath {
    Null,
    Direct(Arc<RwLock<dyn Host>>),
}

impl HydroflowSink for HydroflowPortConfig {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn instantiate(&self, client_path: &SourcePath) -> Result<Box<dyn FnOnce() -> ServerConfig>> {
        let server = self.service.upgrade().unwrap();
        let server_read = server.try_read().unwrap();

        let (bind_type, base_config) = match client_path {
            SourcePath::Direct(client_host) => {
                let client_host_id = client_host.try_read().unwrap().id();

                let server_host_clone = server_read.on.clone();
                let server_host = server_host_clone.try_read().unwrap();

                let client_host_read = if server_host.id() == client_host_id {
                    None
                } else {
                    client_host.try_read().ok()
                };

                let (conn_type, bind_type) =
                    server_host.strategy_as_server(client_host_read.as_deref())?;
                let base_config = ServerConfig::from_strategy(&conn_type, Box::new(self.clone()));
                (bind_type, base_config)
            }

            SourcePath::Null => {
                let strategy_getter: HostStrategyGetter = Box::new(|_| ServerStrategy::Null);
                (strategy_getter, ServerConfig::Null)
            }
        };

        let server = server.clone();
        let merge = self.merge;
        let port = self.port.clone();
        Ok(Box::new(move || {
            let mut server_write = server.try_write().unwrap();
            let bind_type = bind_type(server_write.on.try_write().unwrap().as_any_mut());

            if merge {
                let merge_config = server_write
                    .port_to_bind
                    .entry(port.clone())
                    .or_insert(ServerStrategy::Merge(vec![]));
                let merge_index = if let ServerStrategy::Merge(merge) = merge_config {
                    merge.push(bind_type);
                    merge.len() - 1
                } else {
                    panic!()
                };

                ServerConfig::MergeSelect(Box::new(base_config), merge_index)
            } else {
                assert!(!server_write.port_to_bind.contains_key(&port));
                server_write.port_to_bind.insert(port.clone(), bind_type);
                base_config
            }
        }))
    }

    fn instantiate_reverse(
        &self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<Box<dyn FnOnce(&mut dyn Any) -> ServerStrategy>> {
        let client = self.service.upgrade().unwrap();
        let client_read = client.try_read().unwrap();

        let client_host_id = client_read.on.try_read().unwrap().id();

        let server_host_clone = server_host.clone();
        let server_host = server_host_clone.try_read().unwrap();

        let client_host_clone = client_read.on.clone();
        let client_host_read = if server_host.id() == client_host_id {
            None
        } else {
            client_host_clone.try_read().ok()
        };

        let (conn_type, bind_type) = server_host.strategy_as_server(client_host_read.as_deref())?;
        let client_port = wrap_client_port(ServerConfig::from_strategy(&conn_type, server_sink));

        let client = client.clone();
        let merge = self.merge;
        let port = self.port.clone();
        Ok(Box::new(move |_| {
            let mut client_write = client.try_write().unwrap();

            if merge {
                let merge_config = client_write
                    .port_to_server
                    .entry(port.clone())
                    .or_insert(ServerConfig::Merge(vec![]));

                if let ServerConfig::Merge(merge) = merge_config {
                    merge.push(client_port);
                } else {
                    panic!()
                };
            } else {
                assert!(!client_write.port_to_server.contains_key(&port));
                client_write
                    .port_to_server
                    .insert(port.clone(), client_port);
            };

            let mut server_host = client_write.on.try_write().unwrap();
            bind_type(server_host.as_any_mut())
        }))
    }
}

pub enum ServerConfig {
    Direct(Box<dyn HydroflowServer>),
    Forwarded(Box<dyn HydroflowServer>),
    /// A demux that will be used at runtime to listen to many connections.
    Demux(HashMap<u32, ServerConfig>),
    /// The other side of a demux, with a port to extract the appropriate connection.
    DemuxSelect(Box<ServerConfig>, u32),
    /// A merge that will be used at runtime to combine many connections.
    Merge(Vec<ServerConfig>),
    /// The other side of a merge, with a port to extract the appropriate connection.
    MergeSelect(Box<ServerConfig>, usize),
    Null,
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
        ServerPort::Merge(merge) => {
            let mut forwarded_vec = Vec::new();
            for conn in merge {
                forwarded_vec.push(forward_connection(conn, target).await);
            }
            ServerPort::Merge(forwarded_vec)
        }
        ServerPort::Null => ServerPort::Null,
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

            ServerConfig::Merge(merge) => {
                let mut merge_vec = Vec::new();
                for conn in merge {
                    merge_vec.push(conn.sink_port().await);
                }
                ServerPort::Merge(merge_vec)
            }

            ServerConfig::MergeSelect(underlying, key) => {
                if let ServerPort::Merge(mut mapping) = underlying.sink_port().await {
                    mapping.remove(*key)
                } else {
                    panic!("Expected a merge connection definition")
                }
            }

            ServerConfig::Null => ServerPort::Null,
        }
    }
}
