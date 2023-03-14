use std::sync::{Arc, Weak};

use anyhow::Result;
use async_trait::async_trait;
use hydroflow_cli_integration::ServerOrBound;
use tokio::sync::RwLock;

use super::{
    hydroflow_crate::ports::{ClientPort, HydroflowPortConfig, HydroflowSource},
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
    client_port: Option<ClientPort>,
}

impl CustomClientPort {
    pub fn new(on: Weak<RwLock<CustomService>>) -> Self {
        Self {
            on,
            client_port: None,
        }
    }

    pub async fn server_port(&self) -> ServerOrBound {
        ServerOrBound::Server(self.client_port.as_ref().unwrap().connection_defn().await)
    }
}

impl HydroflowSource for CustomClientPort {
    fn send_to(&mut self, to: HydroflowPortConfig) {
        if let Ok(instantiated) = to.instantiate(&self.on.upgrade().unwrap().try_read().unwrap().on)
        {
            self.client_port = Some(instantiated);
        } else {
            panic!("Custom services cannot be used as the server")
        }
    }
}
