use std::{fmt::Debug, sync::Arc};

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use hydroflow::util::connection::BindType;
use tokio::sync::RwLock;

pub mod deployment;
pub use deployment::Deployment;

pub mod localhost;
pub use localhost::LocalhostHost;

pub mod hydroflow_crate;
pub use hydroflow_crate::HydroflowCrate;

pub struct ResourceBatch {}

impl ResourceBatch {
    async fn provision(&mut self) -> ResourceResult {
        ResourceResult {}
    }
}

pub struct ResourceResult {}

#[async_trait]
pub trait LaunchedBinary: Send + Sync {
    async fn stdin(&self) -> Sender<String>;
    async fn stdout(&self) -> Receiver<String>;
    async fn stderr(&self) -> Receiver<String>;

    async fn exit_code(&self) -> Option<i32>;
}

#[async_trait]
pub trait LaunchedHost: Send + Sync {
    async fn launch_binary(&self, binary: String) -> Arc<RwLock<dyn LaunchedBinary>>;
}

pub enum ConnectionType {
    UnixSocket(
        /// Unique identifier for the host this socket will be on.
        usize,
    ),
    InternalTcpPort(
        /// Unique identifier for the VPC this port will be on.
        usize,
    ),
}

#[async_trait]
pub trait Host: Send + Sync + Debug {
    /// Collect the set of resources that this host needs to run.
    async fn collect_resources(&mut self, resource_batch: &mut ResourceBatch);

    async fn provision(&mut self, resource_result: &ResourceResult) -> Arc<dyn LaunchedHost>;

    fn find_bind_type(&self, connection_from: &dyn Host) -> BindType;

    fn can_connect_to(&self, typ: ConnectionType) -> bool;
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
    /// Collect the set of resources that this service needs to run.
    async fn collect_resources(&mut self, terraform: &mut ResourceBatch);

    async fn deploy(&mut self, terraform_result: &ResourceResult);
    async fn ready(&mut self);
    async fn start(&mut self);

    fn host(&self) -> Arc<RwLock<dyn Host>>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
