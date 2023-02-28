use std::{path::Path, sync::Arc};

use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use hydroflow::util::connection::BindConfig;
use tokio::sync::RwLock;

pub mod deployment;
pub use deployment::Deployment;

pub mod localhost;
pub use localhost::LocalhostHost;

pub mod gcp;
pub use gcp::GCPComputeEngineHost;

pub mod hydroflow_crate;
pub use hydroflow_crate::HydroflowCrate;

pub mod custom_service;
pub use custom_service::CustomService;

pub mod terraform;

pub mod util;

pub struct ResourceBatch {
    pub terraform: terraform::TerraformBatch,
}

impl ResourceBatch {
    fn new() -> ResourceBatch {
        ResourceBatch {
            terraform: terraform::TerraformBatch::default(),
        }
    }

    async fn provision(self) -> ResourceResult {
        ResourceResult {
            terraform: self.terraform.provision().await,
        }
    }
}

pub struct ResourceResult {
    pub terraform: terraform::TerraformResult,
}

#[async_trait]
pub trait LaunchedBinary: Send + Sync {
    async fn stdin(&self) -> Sender<String>;
    async fn stdout(&self) -> Receiver<String>;
    async fn stderr(&self) -> Receiver<String>;

    async fn exit_code(&self) -> Option<i32>;
}

#[async_trait]
pub trait LaunchedHost: Send + Sync {
    /// Identifies a network type that this host can use for connections from the given host.
    fn get_bind_config(&self, bind_type: &BindType) -> BindConfig;

    async fn launch_binary(&self, binary: &Path) -> Result<Arc<RwLock<dyn LaunchedBinary>>>;
}

pub enum BindType {
    UnixSocket,
    InternalTcpPort,
    ExternalTcpPort(u16),
}

pub enum ConnectionType {
    UnixSocket(
        /// Unique identifier for the host this socket will be on.
        usize,
    ),
    InternalTcpPort(
        /// Unique identifier for the VPC this port will be on.
        String,
    ),
}

#[async_trait]
pub trait Host: Send + Sync {
    fn request_port(&mut self, bind_type: &BindType);

    /// Makes requests for physical resources (servers) that this host needs to run.
    fn collect_resources(&self, resource_batch: &mut ResourceBatch);

    /// Connects to the acquired resources and prepares the host to run services.
    async fn provision(&mut self, resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost>;

    /// Identifies a network type that this host can use for connections from the given host.
    fn get_bind_type(&self, connection_from: &dyn Host) -> BindType;

    fn can_connect_to(&self, typ: ConnectionType) -> bool;
}

#[async_trait]
pub trait Service: Send + Sync {
    /// Makes requests for physical resources (servers) that this service needs to run.
    /// This should also perform any "free", non-blocking computations (compilations),
    /// because the `deploy` method will be called after these resources are allocated.
    fn collect_resources(&mut self, resource_batch: &mut ResourceBatch);

    /// Connects to the acquired resources and prepares the service to be launched.
    async fn deploy(&mut self, resource_result: &Arc<ResourceResult>);

    /// Launches the service, which should start listening for incoming network
    /// connections. The service should not start computing at this point.
    async fn ready(&mut self) -> Result<()>;

    /// Starts the service by having it connect to other services and start computations.
    async fn start(&mut self);

    fn host(&self) -> Arc<RwLock<dyn Host>>;
}
