use std::{path::Path, sync::Arc};

use anyhow::Result;
use async_channel::{Receiver, Sender};
use async_process::{Command, Stdio};
use async_trait::async_trait;
use futures::{io::BufReader, AsyncBufReadExt, AsyncRead, AsyncWriteExt, StreamExt};
use hydroflow::util::cli::BindConfig;
use tokio::sync::RwLock;

use super::{
    BindType, ConnectionType, Host, HostTargetType, LaunchedBinary, LaunchedHost, ResourceBatch,
    ResourceResult,
};

struct LaunchedLocalhostBinary {
    child: RwLock<async_process::Child>,
    stdin_sender: Sender<String>,
    stdout_receivers: Arc<RwLock<Vec<Sender<String>>>>,
    stderr_receivers: Arc<RwLock<Vec<Sender<String>>>>,
}

#[async_trait]
impl LaunchedBinary for LaunchedLocalhostBinary {
    async fn stdin(&self) -> Sender<String> {
        self.stdin_sender.clone()
    }

    async fn stdout(&self) -> Receiver<String> {
        let mut receivers = self.stdout_receivers.write().await;
        let (sender, receiver) = async_channel::unbounded::<String>();
        receivers.push(sender);
        receiver
    }

    async fn stderr(&self) -> Receiver<String> {
        let mut receivers = self.stderr_receivers.write().await;
        let (sender, receiver) = async_channel::unbounded::<String>();
        receivers.push(sender);
        receiver
    }

    async fn exit_code(&self) -> Option<i32> {
        self.child
            .write()
            .await
            .try_status()
            .map(|s| s.and_then(|c| c.code()))
            .unwrap_or(None)
    }
}

struct LaunchedLocalhost {}

pub fn create_broadcast<T: AsyncRead + Send + Unpin + 'static>(
    source: T,
    default: impl Fn(String) + Send + 'static,
) -> Arc<RwLock<Vec<Sender<String>>>> {
    let receivers = Arc::new(RwLock::new(Vec::<Sender<String>>::new()));
    let weak_receivers = Arc::downgrade(&receivers);

    tokio::spawn(async move {
        let mut lines = BufReader::new(source).lines();

        while let Some(Result::Ok(line)) = lines.next().await {
            if let Some(receivers) = weak_receivers.upgrade() {
                let mut receivers = receivers.write().await;
                let mut successful_send = false;
                for r in receivers.iter() {
                    successful_send |= r.send(line.clone()).await.is_ok();
                }

                receivers.retain(|r| !r.is_closed());

                if !successful_send {
                    default(line);
                }
            } else {
                break;
            }
        }
    });

    receivers
}

#[async_trait]
impl LaunchedHost for LaunchedLocalhost {
    fn get_bind_config(&self, bind_type: &BindType) -> BindConfig {
        match bind_type {
            BindType::UnixSocket => BindConfig::UnixSocket,
            BindType::InternalTcpPort => BindConfig::TcpPort("127.0.0.1".to_string()),
            BindType::ExternalTcpPort(_) => panic!("Cannot bind to external port"),
        }
    }

    async fn launch_binary(&self, binary: &Path) -> Result<Arc<RwLock<dyn LaunchedBinary>>> {
        let mut child = Command::new(binary)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (stdin_sender, mut stdin_receiver) = async_channel::unbounded::<String>();
        let mut stdin = child.stdin.take().unwrap();
        tokio::spawn(async move {
            while let Some(line) = stdin_receiver.next().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let stdout_receivers = create_broadcast(child.stdout.take().unwrap(), |s| println!("{s}"));
        let stderr_receivers = create_broadcast(child.stderr.take().unwrap(), |s| eprintln!("{s}"));

        Ok(Arc::new(RwLock::new(LaunchedLocalhostBinary {
            child: RwLock::new(child),
            stdin_sender,
            stdout_receivers,
            stderr_receivers,
        })))
    }
}

#[derive(Debug)]
pub struct LocalhostHost {
    pub id: usize,
}

#[async_trait]
impl Host for LocalhostHost {
    fn target_type(&self) -> HostTargetType {
        HostTargetType::Local
    }

    fn request_port(&mut self, _bind_type: &BindType) {}
    fn collect_resources(&self, _resource_batch: &mut ResourceBatch) {}
    fn request_custom_binary(&mut self) {}

    async fn provision(&mut self, _resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {})
    }

    fn get_bind_type(&self, connection_from: &dyn Host) -> BindType {
        if connection_from.can_connect_to(ConnectionType::UnixSocket(self.id)) {
            BindType::UnixSocket
        } else if connection_from
            .can_connect_to(ConnectionType::InternalTcpPort(format!("{}", self.id)))
        {
            BindType::InternalTcpPort
        } else {
            todo!()
        }
    }

    fn can_connect_to(&self, typ: ConnectionType) -> bool {
        match typ {
            ConnectionType::UnixSocket(id) => {
                #[cfg(unix)]
                {
                    self.id == id
                }

                #[cfg(not(unix))]
                {
                    let _ = id;
                    false
                }
            }
            ConnectionType::InternalTcpPort(id) => format!("{}", self.id) == id,
        }
    }
}
