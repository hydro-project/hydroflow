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

pub trait HydroflowSource {
    fn send_to(&mut self, to: HydroflowPortConfig);
}

#[async_trait]
pub trait HydroflowSink: DynClone + Send + Sync {
    async fn get_server_port(&self) -> ServerPort;
    async fn launched_host(&self) -> Arc<dyn LaunchedHost>;
}

#[derive(Clone)]
pub enum HydroflowPortConfig {
    Direct(Weak<RwLock<HydroflowCrate>>, String),
    Demux(HashMap<u32, HydroflowPortConfig>),
}

impl HydroflowPortConfig {
    pub fn instantiate(&self, client_host: &Arc<RwLock<dyn Host>>) -> Result<ClientPort> {
        match self {
            HydroflowPortConfig::Direct(server_weak, server_port) => {
                let server = server_weak.upgrade().unwrap();
                let mut server_write = server.try_write().unwrap();

                let client_host_id = client_host.try_read().unwrap().id();

                let server_host_clone = server_write.on.clone();
                let mut server_host = server_host_clone.try_write().unwrap();

                let client_host_read = if server_host.id() == client_host_id {
                    None
                } else {
                    client_host.try_read().ok()
                };

                let (conn_type, bind_type) =
                    server_host.strategy_as_server(client_host_read.as_deref())?;

                server_write
                    .server_ports
                    .insert(server_port.clone(), bind_type);

                match conn_type {
                    ClientStrategy::UnixSocket(_) | ClientStrategy::InternalTcpPort(_) => {
                        Ok(ClientPort::Direct(Box::new(self.clone())))
                    }
                    ClientStrategy::ForwardedTcpPort(_) => {
                        Ok(ClientPort::Forwarded(Box::new(self.clone())))
                    }
                }
            }
            HydroflowPortConfig::Demux(demux) => {
                let mut instantiated_map = HashMap::new();
                for (key, target) in demux {
                    instantiated_map.insert(*key, target.instantiate(client_host)?);
                }

                Ok(ClientPort::Demux(instantiated_map))
            }
        }
    }

    pub fn instantiate_reverse(
        &self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowSink>,
        wrap_client_port: &dyn Fn(ClientPort) -> ClientPort,
    ) -> Result<ServerStrategy> {
        match self {
            HydroflowPortConfig::Direct(client_weak, client_port) => {
                let client = client_weak.upgrade().unwrap();
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

                let (conn_type, bind_type) =
                    server_host.strategy_as_server(client_host_read.as_deref())?;

                client_write.client_port.insert(
                    client_port.clone(),
                    wrap_client_port(match conn_type {
                        ClientStrategy::UnixSocket(_) | ClientStrategy::InternalTcpPort(_) => {
                            ClientPort::Direct(server_sink)
                        }
                        ClientStrategy::ForwardedTcpPort(_) => ClientPort::Forwarded(server_sink),
                    }),
                );

                Ok(bind_type)
            }
            HydroflowPortConfig::Demux(demux) => {
                let mut instantiated_map = HashMap::new();
                for (key, target) in demux {
                    instantiated_map.insert(
                        *key,
                        target.instantiate_reverse(
                            server_host,
                            dyn_clone::clone_box(&*server_sink),
                            // the parent wrapper selects the demux port for the parent defn, so do that first
                            &|p| ClientPort::DemuxSelect(Box::new(wrap_client_port(p)), *key),
                        )?,
                    );
                }

                Ok(ServerStrategy::Demux(instantiated_map))
            }
        }
    }
}

impl HydroflowSource for HydroflowPortConfig {
    fn send_to(&mut self, to: HydroflowPortConfig) {
        if let HydroflowPortConfig::Direct(from, from_port) = self {
            let from = from.upgrade().unwrap();
            let mut from_write = from.try_write().unwrap();
            from_write
                .add_client_port(&from, from_port.clone(), to)
                .unwrap();
        } else {
            panic!("Can only send to a direct port")
        }
    }
}

#[async_trait]
impl HydroflowSink for HydroflowPortConfig {
    async fn get_server_port(&self) -> ServerPort {
        match self {
            HydroflowPortConfig::Direct(server_weak, server_port) => {
                let server = server_weak.upgrade().unwrap();
                let server = server.read().await;
                server.server_defns.get(server_port).unwrap().clone()
            }
            _ => panic!("Can only get server port from a direct port"),
        }
    }

    async fn launched_host(&self) -> Arc<dyn LaunchedHost> {
        match self {
            HydroflowPortConfig::Direct(server_weak, _) => {
                let server = server_weak.upgrade().unwrap();
                let server_read = server.read().await;
                server_read.launched_host.as_ref().unwrap().clone()
            }
            _ => panic!("Can only get server port from a direct port"),
        }
    }
}

pub enum ClientPort {
    Direct(Box<dyn HydroflowSink>),
    Forwarded(Box<dyn HydroflowSink>),
    Demux(HashMap<u32, ClientPort>),
    DemuxSelect(Box<ClientPort>, u32),
}

#[async_recursion]
async fn forward_connection(conn: &ServerPort, target: &dyn LaunchedHost) -> ServerPort {
    match conn {
        ServerPort::UnixSocket(_) => panic!("Expected a TCP port to be forwarded"),
        ServerPort::TcpPort(addr) => {
            ServerPort::TcpPort(target.forward_port(addr.port()).await.unwrap())
        }
        ServerPort::Demux(demux) => {
            let mut forwarded_map = HashMap::new();
            for (key, conn) in demux {
                forwarded_map.insert(*key, forward_connection(conn, target).await);
            }
            ServerPort::Demux(forwarded_map)
        }
    }
}

impl ClientPort {
    #[async_recursion]
    pub async fn connection_defn(&self) -> ServerPort {
        match self {
            ClientPort::Direct(sink) => sink.get_server_port().await,

            ClientPort::Forwarded(sink) => {
                forward_connection(
                    &sink.get_server_port().await,
                    sink.launched_host().await.as_ref(),
                )
                .await
            }

            ClientPort::Demux(demux) => {
                let mut demux_map = HashMap::new();
                for (key, conn) in demux {
                    demux_map.insert(*key, conn.connection_defn().await);
                }
                ServerPort::Demux(demux_map)
            }

            ClientPort::DemuxSelect(underlying, key) => {
                if let ServerPort::Demux(mut mapping) = underlying.connection_defn().await {
                    mapping.remove(key).unwrap()
                } else {
                    panic!("Expected a demux connection definition")
                }
            }
        }
    }
}
