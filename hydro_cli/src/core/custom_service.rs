use std::sync::{Arc, Weak};

use anyhow::{bail, Result};
use async_trait::async_trait;
use hydroflow_cli_integration::ServerOrBound;
use tokio::sync::RwLock;

use super::{
    hydroflow_crate::ports::{HydroflowServer, HydroflowSink, HydroflowSource, ServerConfig},
    Host, LaunchedHost, ResourceBatch, ResourceResult, ServerStrategy, Service,
};

/// Represents an unknown, third-party service that is not part of the Hydroflow ecosystem.
pub struct CustomService {
    on: Arc<RwLock<dyn Host>>,

    /// The ports that the service wishes to expose to the public internet.
    external_ports: Vec<u16>,

    launched_host: Option<Arc<dyn LaunchedHost>>,
}

impl CustomService {
    pub fn new(on: Arc<RwLock<dyn Host>>, external_ports: Vec<u16>) -> Self {
        Self {
            on,
            external_ports,
            launched_host: None,
        }
    }
}

#[async_trait]
impl Service for CustomService {
    fn collect_resources(&mut self, _resource_batch: &mut ResourceBatch) {
        let mut host = self
            .on
            .try_write()
            .expect("No one should be reading/writing the host while resources are collected");

        for port in self.external_ports.iter() {
            host.request_port(&ServerStrategy::ExternalTcpPort(*port));
        }
    }

    async fn deploy(&mut self, resource_result: &Arc<ResourceResult>) {
        let mut host_write = self.on.write().await;
        let launched = host_write.provision(resource_result);
        self.launched_host = Some(launched.await);
    }

    async fn ready(&mut self) -> Result<()> {
        Ok(())
    }

    async fn start(&mut self) {}
}

pub struct CustomClientPort {
    pub on: Weak<RwLock<CustomService>>,
    client_port: RwLock<Option<ServerConfig>>,
}

impl CustomClientPort {
    pub fn new(on: Weak<RwLock<CustomService>>) -> Self {
        Self {
            on,
            client_port: RwLock::new(None),
        }
    }

    pub async fn server_port(&self) -> ServerOrBound {
        ServerOrBound::Server(
            self.client_port
                .read()
                .await
                .as_ref()
                .unwrap()
                .sink_port()
                .await,
        )
    }
}

impl HydroflowSource for CustomClientPort {
    fn send_to(&mut self, to: &dyn HydroflowSink) {
        if let Ok(instantiated) = to.instantiate(&self.on.upgrade().unwrap().try_read().unwrap().on)
        {
            *self.client_port.blocking_write() = Some(instantiated);
        } else {
            panic!("Custom services cannot be used as the server")
        }
    }
}

#[async_trait]
impl HydroflowSink for CustomClientPort {
    fn instantiate(&self, _client_host: &Arc<RwLock<dyn Host>>) -> Result<ServerConfig> {
        bail!("Custom services cannot be used as the server")
    }

    fn instantiate_reverse(
        &self,
        server_host: &Arc<RwLock<dyn Host>>,
        server_sink: Box<dyn HydroflowServer>,
        wrap_client_port: &dyn Fn(ServerConfig) -> ServerConfig,
    ) -> Result<ServerStrategy> {
        let client = self.on.upgrade().unwrap();
        let client_read = client.try_read().unwrap();

        let client_host_id = client_read.on.try_read().unwrap().id();

        let server_host_clone = server_host.clone();
        let mut server_host = server_host_clone.try_write().unwrap();

        let client_host_clone = client_read.on.clone();
        let client_host_read = if server_host.id() == client_host_id {
            None
        } else {
            client_host_clone.try_read().ok()
        };

        let (conn_type, bind_type) = server_host.strategy_as_server(client_host_read.as_deref())?;

        *self.client_port.blocking_write() = Some(wrap_client_port(ServerConfig::from_strategy(
            &conn_type,
            server_sink,
        )));

        Ok(bind_type)
    }
}
