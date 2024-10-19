#[cfg(unix)]
use std::sync::Arc;

use async_trait::async_trait;
use futures::TryStreamExt;
use kube::api::AttachedProcess;
use tokio::sync::{mpsc, oneshot};
use std::sync::Mutex;
use anyhow::Error;

use tokio::io::AsyncWriteExt;

use crate::util::prioritized_broadcast;
use crate::LaunchedBinary;

// pub struct LaunchedPodBinary {
//     stdin_sender: Sender<String>,
//     stdout_cli_receivers: Arc<RwLock<Option<tokio::sync::oneshot::Sender<String>>>>,
//     stdout_receivers: Arc<RwLock<Vec<Sender<String>>>>,
//     stderr_receivers: Arc<RwLock<Vec<Sender<String>>>>,
// }

pub struct LaunchedPodBinary {
    stdin_sender: mpsc::UnboundedSender<String>,
    stdout_deploy_receivers: Arc<Mutex<Option<oneshot::Sender<String>>>>,
    stdout_receivers: Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>,
    stderr_receivers: Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>,
}

impl LaunchedPodBinary {
    pub fn new(launched_pod_binary: &mut AttachedProcess, id: String) -> Self {
        // Create streams for stdout and stdin for the running binary in the pod

        let launch_binary_out = tokio_util::io::ReaderStream::new(launched_pod_binary.stdout().unwrap());
        let launch_binary_err = tokio_util::io::ReaderStream::new(launched_pod_binary.stderr().unwrap());
        let mut stdin = launched_pod_binary.stdin().unwrap();


        let (stdin_sender, mut stdin_receiver) = mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            while let Some(line) = stdin_receiver.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }

                stdin.flush().await.ok();
            }
        });

        let id_clone = id.clone();
        let (stdout_deploy_receivers, stdout_receivers) = prioritized_broadcast(
            launch_binary_out.map_ok(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            move |s| println!("[{id_clone}] {s}"),
        );
        let (_, stderr_receivers) = prioritized_broadcast(
            launch_binary_err.map_ok(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            move |s| eprintln!("[{id}] {s}"),
        );

        Self {
            stdin_sender,
            stdout_deploy_receivers,
            stdout_receivers,
            stderr_receivers,
        }
    }
}

#[async_trait]
impl LaunchedBinary for LaunchedPodBinary {
    fn stdin(&self) -> mpsc::UnboundedSender<String> {
        self.stdin_sender.clone()
    }

    fn deploy_stdout(&self) -> oneshot::Receiver<String> {
        let mut receivers = self.stdout_deploy_receivers.lock().unwrap();

        if receivers.is_some() {
            panic!("Only one deploy stdout receiver is allowed at a time");
        }

        let (sender, receiver) = oneshot::channel::<String>();
        *receivers = Some(sender);
        receiver
    }

    fn stdout(&self) -> mpsc::UnboundedReceiver<String> {
        let mut receivers = self.stdout_receivers.lock().unwrap();
        let (sender, receiver) = mpsc::unbounded_channel::<String>();
        receivers.push(sender);
        receiver
    }

    fn stderr(&self) -> mpsc::UnboundedReceiver<String> {
        let mut receivers = self.stderr_receivers.lock().unwrap();
        let (sender, receiver) = mpsc::unbounded_channel::<String>();
        receivers.push(sender);
        receiver
    }

    // returns exit code when the hydroflow program finishes
    fn exit_code(&self) -> Option<i32> {
        None
    }

    // waits for the hydroflow program to finish
    async fn wait(&mut self) -> Result<i32, Error> {
        Ok(1)
    }

    // waits for the hydroflow program to finish
    async fn stop(&mut self) -> Result<(), Error> {
        // Implement the logic to stop the hydroflow program
        // For now, we will return Ok(()) to indicate success
        Ok(())
    }
}
