use std::{any::Any, fmt::Debug, sync::Arc};

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use hydroflow::util::connection::ConnectionPipe;
use tokio::sync::RwLock;

pub mod deployment;
pub use deployment::Deployment;

pub mod localhost;
pub use localhost::LocalhostHost;

pub mod hydroflow_crate;
pub use hydroflow_crate::HydroflowCrate;

pub struct TerraformBatch {}

impl TerraformBatch {
    async fn provision(&mut self) -> TerraformResult {
        TerraformResult {}
    }
}

pub struct TerraformResult {}

#[async_trait]
pub trait LaunchedBinary: Send + Sync {
    fn stdin(&self) -> Sender<String>;
    fn stdout(&self) -> Receiver<String>;
    fn stderr(&self) -> Receiver<String>;

    fn exit_code(&self) -> Option<i32>;
}

#[async_trait]
pub trait LaunchedHost: Send + Sync {
    async fn launch_binary(&self, binary: String) -> Arc<RwLock<dyn LaunchedBinary>>;
}

pub enum ConnectionType {
    UnixSocket(usize),
}

#[async_trait]
pub trait Host: Send + Sync + Debug {
    /// Collect the set of resources that this host needs to run.
    async fn collect_resources(&mut self, terraform: &mut TerraformBatch);

    async fn provision(&mut self, terraform_result: &TerraformResult) -> Arc<dyn LaunchedHost>;

    async fn allocate_pipe(
        &self,
        client: Arc<RwLock<dyn Host>>,
    ) -> (ConnectionPipe, Box<dyn Any + Send + Sync>);

    fn can_connect_to(&self, typ: ConnectionType) -> bool;
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
    /// Collect the set of resources that this service needs to run.
    async fn collect_resources(&mut self, terraform: &mut TerraformBatch);

    async fn deploy(&mut self, terraform_result: &TerraformResult);
    async fn ready(&mut self);
    async fn start(&mut self);

    fn host(&self) -> Arc<RwLock<dyn Host>>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
