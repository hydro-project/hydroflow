#[cfg(unix)]
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use kube::api::AttachedProcess;
use tokio::sync::RwLock;

use tokio::io::AsyncWriteExt;

use crate::util::prioritized_broadcast;
use crate::LaunchedBinary;

pub struct LaunchedPodBinary {
    stdin_sender: Sender<String>,
    stdout_cli_receivers: Arc<RwLock<Option<tokio::sync::oneshot::Sender<String>>>>,
    stdout_receivers: Arc<RwLock<Vec<Sender<String>>>>,
    stderr_receivers: Arc<RwLock<Vec<Sender<String>>>>,
}

impl LaunchedPodBinary {
    pub fn new(launched_pod_binary: &mut AttachedProcess, id: String) -> Self {
        // Create streams for stdout and stdin for the running binary in the pod

        let launch_binary_out = tokio_util::io::ReaderStream::new(launched_pod_binary.stdout().unwrap());
        let launch_binary_err = tokio_util::io::ReaderStream::new(launched_pod_binary.stderr().unwrap());
        let mut stdin = launched_pod_binary.stdin().unwrap();


        let (stdin_sender, mut stdin_receiver) = async_channel::unbounded::<String>();
        tokio::spawn(async move {
            while let Some(line) = stdin_receiver.next().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }

                stdin.flush().await.ok();
            }
        });

        let id_clone = id.clone();
        let (stdout_cli_receivers, stdout_receivers) = prioritized_broadcast(
            launch_binary_out.map_ok(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            move |s| println!("[{id_clone}] {s}"),
        );
        let (_, stderr_receivers) = prioritized_broadcast(
            launch_binary_err.map_ok(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            move |s| eprintln!("[{id}] {s}"),
        );

        Self {
            stdin_sender,
            stdout_cli_receivers,
            stdout_receivers,
            stderr_receivers,
        }
    }
}

#[async_trait]
impl LaunchedBinary for LaunchedPodBinary {
    async fn stdin(&self) -> Sender<String> {
        self.stdin_sender.clone()
    }

    async fn cli_stdout(&self) -> tokio::sync::oneshot::Receiver<String> {
        let mut receivers = self.stdout_cli_receivers.write().await;

        if receivers.is_some() {
            panic!("Only one CLI stdout receiver is allowed at a time");
        }

        let (sender, receiver) = tokio::sync::oneshot::channel::<String>();
        *receivers = Some(sender);
        receiver
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

    // returns exit code when the hydroflow program finishes
    async fn exit_code(&self) -> Option<i32> {
        None
    }

    // waits for the hydroflow program to finish
    async fn wait(&mut self) -> Option<i32> {
        None
    }
}
