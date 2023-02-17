use std::{fmt::Debug, path::PathBuf, sync::Arc};

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use tokio::sync::RwLock;

pub mod deployment;
pub use deployment::Deployment;

pub mod localhost;
pub use localhost::LocalhostHost;

pub mod hydroflow_crate;
pub use hydroflow_crate::HydroflowCrate;

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

#[derive(Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
}

pub enum ConnectionType {
    UnixSocket(usize),
}

#[async_trait]
pub trait Host: Send + Sync + Debug {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost>;

    async fn allocate_pipe(&self, client: Arc<RwLock<dyn Host>>) -> ConnectionPipe;

    fn can_connect_to(&self, typ: ConnectionType) -> bool;
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
    async fn deploy(&mut self);
    async fn ready(&mut self);
    async fn start(&mut self);

    fn host(&self) -> Arc<RwLock<dyn Host>>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
