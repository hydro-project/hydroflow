use std::sync::Arc;

use async_channel::{Receiver, Sender};
use async_process::{Command, Stdio};
use async_trait::async_trait;
use futures::{io::BufReader, AsyncBufReadExt, AsyncRead, AsyncWriteExt, StreamExt};
use hydroflow::util::connection::BindType;
use tokio::sync::RwLock;

use super::{ConnectionType, Host, LaunchedBinary, LaunchedHost, ResourceBatch, ResourceResult};

struct LaunchedLocalhostBinary {
    child: RwLock<async_process::Child>,
    stdin_channel: Sender<String>,
    stdout_receivers: Arc<RwLock<Vec<Sender<String>>>>,
    stderr_receivers: Arc<RwLock<Vec<Sender<String>>>>,
}

#[async_trait]
impl LaunchedBinary for LaunchedLocalhostBinary {
    async fn stdin(&self) -> Sender<String> {
        self.stdin_channel.clone()
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

fn create_broadcast<T: AsyncRead + Send + Unpin + 'static>(
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
    async fn launch_binary(&self, binary: String) -> Arc<RwLock<dyn LaunchedBinary>> {
        let mut child = Command::new(binary)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

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

        Arc::new(RwLock::new(LaunchedLocalhostBinary {
            child: RwLock::new(child),
            stdin_channel: stdin_sender,
            stdout_receivers,
            stderr_receivers,
        }))
    }
}

#[derive(Debug)]
pub struct LocalhostHost {
    pub id: usize,
}

#[async_trait]
impl Host for LocalhostHost {
    async fn collect_resources(&mut self, _terraform: &mut ResourceBatch) {}

    async fn provision(&mut self, _terraform_result: &ResourceResult) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {})
    }

    fn find_bind_type(&self, connection_from: &dyn Host) -> BindType {
        if connection_from.can_connect_to(ConnectionType::UnixSocket(self.id)) {
            BindType::UnixSocket
        } else if connection_from.can_connect_to(ConnectionType::InternalTcpPort(self.id)) {
            BindType::TcpPort("127.0.0.1".to_string())
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
                    false
                }
            }
            ConnectionType::InternalTcpPort(id) => self.id == id,
        }
    }
}
