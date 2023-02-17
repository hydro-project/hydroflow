use std::{path::PathBuf, sync::Arc};

use async_channel::{Receiver, Sender};
use async_process::{Command, Stdio};
use async_trait::async_trait;
use futures::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, StreamExt};
use tokio::sync::RwLock;

use super::{
    ConnectionPipe, ConnectionType, Host, LaunchedBinary, LaunchedHost, TerraformBatch,
    TerraformResult,
};

struct LaunchedLocalhostBinary {
    child: RwLock<async_process::Child>,
    stdin_channel: Sender<String>,
    stdout_channel: Receiver<String>,
    stderr_channel: Receiver<String>,
}

#[async_trait]
impl LaunchedBinary for LaunchedLocalhostBinary {
    fn stdin(&self) -> Sender<String> {
        self.stdin_channel.clone()
    }

    fn stdout(&self) -> Receiver<String> {
        self.stdout_channel.clone()
    }

    fn stderr(&self) -> Receiver<String> {
        self.stderr_channel.clone()
    }

    fn exit_code(&self) -> Option<i32> {
        self.child
            .blocking_write()
            .try_status()
            .map(|s| s.and_then(|c| c.code()))
            .unwrap_or(None)
    }
}

struct LaunchedLocalhost {}

#[async_trait]
impl LaunchedHost for LaunchedLocalhost {
    async fn launch_binary(&self, binary: String) -> Arc<RwLock<dyn LaunchedBinary>> {
        let mut child = Command::new(binary)
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

        let (stdout_sender, stdout_receiver) = async_channel::unbounded();
        let stdout = child.stdout.take().unwrap();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();

            while let Some(line) = lines.next().await {
                if stdout_sender.send(line.unwrap()).await.is_err() {
                    break;
                }
            }
        });

        let (stderr_sender, stderr_receiver) = async_channel::unbounded();
        let stderr = child.stderr.take().unwrap();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();

            while let Some(line) = lines.next().await {
                if stderr_sender.send(line.unwrap()).await.is_err() {
                    break;
                }
            }
        });

        Arc::new(RwLock::new(LaunchedLocalhostBinary {
            child: RwLock::new(child),
            stdin_channel: stdin_sender,
            stdout_channel: stdout_receiver,
            stderr_channel: stderr_receiver,
        }))
    }
}

#[derive(Debug)]
pub struct LocalhostHost {
    pub id: usize,
}

#[async_trait]
impl Host for LocalhostHost {
    async fn collect_resources(&mut self, _terraform: &mut TerraformBatch) {}

    async fn provision(&mut self, _terraform_result: &TerraformResult) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {})
    }

    async fn allocate_pipe(&self, client: Arc<RwLock<dyn Host>>) -> ConnectionPipe {
        if client
            .read()
            .await
            .can_connect_to(ConnectionType::UnixSocket(self.id))
        {
            // TODO(allocate a random file)
            ConnectionPipe::UnixSocket(PathBuf::from("/tmp/hydroflow.sock"))
        } else {
            todo!()
        }
    }

    fn can_connect_to(&self, typ: ConnectionType) -> bool {
        match typ {
            ConnectionType::UnixSocket(id) => self.id == id,
        }
    }
}
