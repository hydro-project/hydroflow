use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use async_trait::async_trait;
use tokio::sync::RwLock;

pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
    pub services: Vec<Arc<RwLock<dyn Service>>>,
}

impl Deployment {
    pub async fn deploy(&mut self) {
        for host in self.hosts.iter_mut() {
            host.write().await.provision().await;
        }

        for service in self.services.iter_mut() {
            service.write().await.deploy().await;
        }
    }

    pub fn add_host<T: Host + 'static>(&mut self, host: T) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(host));
        self.hosts.push(arc.clone());
        arc
    }

    pub fn add_service<T: Service + 'static>(&mut self, service: T) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(service));
        self.services.push(arc.clone());
        arc
    }
}

enum NetworkType {
    Localhost,
    PublicIP,
}

enum BuildTask {
    HydroflowCrate(String),
    // DockerImage(String),
}

enum ProvisioningTask {
    // DockerContainer(...)
}

#[async_trait]
pub trait LaunchedHost: Send + Sync {
    async fn launch_binary(&self, binary: String);
}

struct LaunchedLocalhost {
    // ...
}

#[async_trait]
impl LaunchedHost for LaunchedLocalhost {
    async fn launch_binary(&self, binary: String) {
        todo!("launch binary on host")
    }
}

#[async_trait]
pub trait Host: Send + Sync {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost>;
}

pub struct LocalhostHost {
    // ...
}

#[async_trait]
impl Host for LocalhostHost {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {
            // ...
        })
    }
}

#[async_trait]
pub trait Service: Send + Sync {
    async fn deploy(&mut self);

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct HydroflowCrate {
    pub src: String,
    pub on: Arc<RwLock<dyn Host>>,

    // actually HydroflowCrates
    pub outgoing_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,
    pub incoming_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,
}

#[async_trait]
impl Service for HydroflowCrate {
    async fn deploy(&mut self) {
        let launched = self.on.write().await.provision().await;
        launched.launch_binary(self.src.clone()).await;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
