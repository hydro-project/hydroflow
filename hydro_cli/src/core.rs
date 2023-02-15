use std::sync::Arc;

use async_trait::async_trait;

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
        todo!()
    }
}

#[async_trait]
pub trait Host: Send + Sync {
    async fn provision(&self) -> Arc<dyn LaunchedHost>;
}

pub struct LocalhostHost {
    // ...
}

#[async_trait]
impl Host for LocalhostHost {
    async fn provision(&self) -> Arc<dyn LaunchedHost> {
        dbg!("provisioning localhost");
        Arc::new(LaunchedLocalhost {
            // ...
        })
    }
}

#[async_trait]
pub trait Service: Send + Sync {
    async fn deploy(&self);
}

pub struct HydroflowCrate {
    src: String,
    on: Arc<dyn Host>,
}

#[async_trait]
impl Service for HydroflowCrate {
    async fn deploy(&self) {
        let launched = self.on.provision().await;
        launched.launch_binary(self.src.clone()).await;
    }
}
