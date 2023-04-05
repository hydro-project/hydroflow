use super::HydroflowCrate;
use crate::core::{ClientStrategy, Host, HostStrategyGetter, LaunchedHost, ServerStrategy};

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use dyn_clone::DynClone;

use std::any::Any;
use std::collections::HashMap;

use std::ops::Deref;
use std::sync::{Arc, Weak};
use tokio::sync::RwLock;

use hydroflow_cli_integration::ServerPort;

pub trait HydroflowSource: Send + Sync {
    fn source_path(&self) -> SourcePath;
    fn record_server_config(&mut self, config: ServerConfig);
    fn send_to(&mut self, to: &mut dyn HydroflowSink);
}

#[async_trait]
pub trait HydroflowServer: DynClone + Send + Sync + std::fmt::Debug {
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
        server_sink: Arc<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ReverseSinkInstantiator>;
}

pub struct MuxSource {
    pub mux: HashMap<u32, Arc<RwLock<dyn HydroflowSource>>>,
}

impl HydroflowSource for MuxSource {
    fn source_path(&self) -> SourcePath {
        SourcePath::Mux(
            self.mux
                .iter()
                .map(|(k, v)| (*k, v.try_read().unwrap().source_path()))
                .collect(),
        )
    }

    fn record_server_config(&mut self, config: ServerConfig) {
        if let ServerConfig::Mux(mut mux) = config {
            for (key, target) in &self.mux {
                let mut target = target.try_write().unwrap();
                dbg!("recording mux select!");
                target.record_server_config(ServerConfig::MuxSelect(
                    Box::new(mux.remove(key).unwrap()),
                    *key,
                ));
            }
        }
    }

    fn send_to(&mut self, to: &mut dyn HydroflowSink) {
        let instantiated = to.instantiate(&self.source_path()).unwrap();
        let config = instantiated();
        self.record_server_config(config);
    }
}

pub struct NullSourceSink;

impl HydroflowSource for NullSourceSink {
    fn source_path(&self) -> SourcePath {
        SourcePath::Null
    }

    fn send_to(&mut self, to: &mut dyn HydroflowSink) {
        to.instantiate(&SourcePath::Null).unwrap()();
    }

    fn record_server_config(&mut self, _config: ServerConfig) {}
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
        _server_sink: Arc<dyn HydroflowServer>,
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
        server_sink: Arc<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<Box<dyn FnOnce(&mut dyn Any) -> ServerStrategy>> {
        let mut thunk_map = HashMap::new();
        for (key, target) in &self.demux {
            thunk_map.insert(
                *key,
                target.try_write().unwrap().instantiate_reverse(
                    server_host,
                    server_sink.clone(),
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

#[derive(Clone, Debug)]
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
    fn source_path(&self) -> SourcePath {
        SourcePath::Direct(
            self.service
                .upgrade()
                .unwrap()
                .try_read()
                .unwrap()
                .on
                .clone(),
        )
    }

    fn send_to(&mut self, sink: &mut dyn HydroflowSink) {
        let from = self.service.upgrade().unwrap();

        let forward_res =
            sink.instantiate(&SourcePath::Direct(from.try_read().unwrap().on.clone()));
        if let Ok(instantiated) = forward_res {
            self.record_server_config(instantiated());
        } else {
            drop(forward_res);
            let mut from_write = from.try_write().unwrap();
            let instantiated = sink
                .instantiate_reverse(
                    &from_write.on,
                    Arc::new(HydroflowPortConfig {
                        service: Arc::downgrade(&from),
                        port: self.port.clone(),
                        merge: false,
                    }),
                    &|p| p,
                )
                .unwrap();

            assert!(!from_write.port_to_bind.contains_key(&self.port));
            from_write
                .port_to_bind
                .insert(self.port.clone(), instantiated(sink.as_any_mut()));
        }
    }

    fn record_server_config(&mut self, config: ServerConfig) {
        let from = self.service.upgrade().unwrap();
        let mut from_write = from.try_write().unwrap();

        // TODO(shadaj): if already in this map, we want to broadcast
        assert!(!from_write.port_to_server.contains_key(&self.port));
        from_write.port_to_server.insert(self.port.clone(), config);
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
    Mux(HashMap<u32, SourcePath>),
}

impl SourcePath {
    fn plan<T: HydroflowServer + Clone + 'static>(
        &self,
        server: &T,
        server_host: &dyn Host,
    ) -> Result<(HostStrategyGetter, ServerConfig)> {
        match self {
            SourcePath::Direct(client_host) => {
                let client_host = client_host.try_read().unwrap();
                let (conn_type, bind_type) = server_host.strategy_as_server(client_host.deref())?;
                let base_config = ServerConfig::from_strategy(&conn_type, Arc::new(server.clone()));
                Ok((bind_type, base_config))
            }

            SourcePath::Mux(mux) => {
                let mut strategy_getter_map = HashMap::new();
                let mut base_config_map = HashMap::new();

                for (key, target) in mux {
                    let (bind_type, base_config) = target.plan(server, server_host)?;
                    strategy_getter_map.insert(*key, bind_type);
                    base_config_map.insert(*key, base_config);
                }

                let strategy_getter: HostStrategyGetter = Box::new(move |server_host| {
                    let applied = strategy_getter_map
                        .drain()
                        .map(|(k, v)| (k, v(server_host)))
                        .collect::<HashMap<_, _>>();
                    ServerStrategy::Mux(applied)
                });

                Ok((strategy_getter, ServerConfig::Mux(base_config_map)))
            }

            SourcePath::Null => {
                let strategy_getter: HostStrategyGetter = Box::new(|_| ServerStrategy::Null);
                Ok((strategy_getter, ServerConfig::Null))
            }
        }
    }
}

impl HydroflowSink for HydroflowPortConfig {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn instantiate(&self, client_path: &SourcePath) -> Result<Box<dyn FnOnce() -> ServerConfig>> {
        let server = self.service.upgrade().unwrap();
        let server_read = server.try_read().unwrap();

        let server_host_clone = server_read.on.clone();
        let server_host = server_host_clone.try_read().unwrap();

        let (bind_type, base_config) = client_path.plan(self, server_host.deref())?;

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
        server_sink: Arc<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<Box<dyn FnOnce(&mut dyn Any) -> ServerStrategy>> {
        let client = self.service.upgrade().unwrap();
        let client_read = client.try_read().unwrap();

        let server_host_clone = server_host.clone();
        let server_host = server_host_clone.try_read().unwrap();

        let (conn_type, bind_type) =
            server_host.strategy_as_server(client_read.on.try_read().unwrap().deref())?;
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

#[derive(Clone, Debug)]
pub enum ServerConfig {
    Direct(Arc<dyn HydroflowServer>),
    Forwarded(Arc<dyn HydroflowServer>),
    /// A demux that will be used at runtime to listen to many connections.
    Demux(HashMap<u32, ServerConfig>),
    /// The other side of a demux, with a port to extract the appropriate connection.
    DemuxSelect(Box<ServerConfig>, u32),
    /// A merge that will be used at runtime to combine many connections.
    Merge(Vec<ServerConfig>),
    /// The other side of a merge, with a port to extract the appropriate connection.
    MergeSelect(Box<ServerConfig>, usize),
    Mux(HashMap<u32, ServerConfig>),
    MuxSelect(Box<ServerConfig>, u32),
    Null,
}

impl ServerConfig {
    pub fn from_strategy(
        strategy: &ClientStrategy,
        server: Arc<dyn HydroflowServer>,
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
        ServerPort::Mux(mux) => {
            let mut forwarded_map = HashMap::new();
            for (key, conn) in mux {
                forwarded_map.insert(*key, forward_connection(conn, target).await);
            }
            ServerPort::Mux(forwarded_map)
        }
        ServerPort::Null => ServerPort::Null,
    }
}

impl ServerConfig {
    #[async_recursion]
    pub async fn load_instantiated(&self) -> ServerPort {
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
                    demux_map.insert(*key, conn.load_instantiated().await);
                }
                ServerPort::Demux(demux_map)
            }

            ServerConfig::DemuxSelect(underlying, key) => {
                if let ServerPort::Demux(mut mapping) = underlying.load_instantiated().await {
                    mapping.remove(key).unwrap()
                } else {
                    panic!("Expected a demux connection definition")
                }
            }

            ServerConfig::Merge(merge) => {
                let mut merge_vec = Vec::new();
                for conn in merge {
                    merge_vec.push(conn.load_instantiated().await);
                }
                ServerPort::Merge(merge_vec)
            }

            ServerConfig::MergeSelect(underlying, key) => {
                if let ServerPort::Merge(mut mapping) = underlying.load_instantiated().await {
                    mapping.remove(*key)
                } else {
                    panic!("Expected a merge connection definition")
                }
            }

            ServerConfig::Mux(mux) => {
                dbg!(mux);
                let mut mux_map = HashMap::new();
                for (key, conn) in mux {
                    mux_map.insert(*key, conn.load_instantiated().await);
                }
                ServerPort::Mux(mux_map)
            }

            ServerConfig::MuxSelect(underlying, key) => {
                if let ServerPort::Mux(mut mapping) = underlying.load_instantiated().await {
                    mapping.remove(key).unwrap()
                } else {
                    panic!("Expected a mux connection definition")
                }
            }

            ServerConfig::Null => ServerPort::Null,
        }
    }
}
