#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use futures::io::BufReader;
use futures::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};

use crate::util::prioritized_broadcast;
use crate::LaunchedBinary;

pub struct LaunchedLocalhostBinary {
    child: Mutex<async_process::Child>,
    stdin_sender: mpsc::UnboundedSender<String>,
    stdout_cli_receivers: Arc<Mutex<Option<oneshot::Sender<String>>>>,
    stdout_receivers: Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>,
    stderr_receivers: Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>,
}

#[cfg(unix)]
impl Drop for LaunchedLocalhostBinary {
    fn drop(&mut self) {
        let mut child = self.child.lock().unwrap();

        if let Ok(Some(_)) = child.try_status() {
            return;
        }

        let pid = child.id();
        if let Err(e) = nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid as i32),
            nix::sys::signal::SIGTERM,
        ) {
            eprintln!("Failed to SIGTERM process {}: {}", pid, e);
        }
    }
}

impl LaunchedLocalhostBinary {
    pub fn new(mut child: async_process::Child, id: String) -> Self {
        let (stdin_sender, mut stdin_receiver) = mpsc::unbounded_channel::<String>();
        let mut stdin = child.stdin.take().unwrap();
        tokio::spawn(async move {
            while let Some(line) = stdin_receiver.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }

                stdin.flush().await.ok();
            }
        });

        let id_clone = id.clone();
        let (stdout_cli_receivers, stdout_receivers) = prioritized_broadcast(
            BufReader::new(child.stdout.take().unwrap()).lines(),
            move |s| println!("[{id_clone}] {s}"),
        );
        let (_, stderr_receivers) = prioritized_broadcast(
            BufReader::new(child.stderr.take().unwrap()).lines(),
            move |s| eprintln!("[{id}] {s}"),
        );

        Self {
            child: Mutex::new(child),
            stdin_sender,
            stdout_cli_receivers,
            stdout_receivers,
            stderr_receivers,
        }
    }
}

#[async_trait]
impl LaunchedBinary for LaunchedLocalhostBinary {
    fn stdin(&self) -> mpsc::UnboundedSender<String> {
        self.stdin_sender.clone()
    }

    fn cli_stdout(&self) -> oneshot::Receiver<String> {
        let mut receivers = self.stdout_cli_receivers.lock().unwrap();

        if receivers.is_some() {
            panic!("Only one CLI stdout receiver is allowed at a time");
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

    fn exit_code(&self) -> Option<i32> {
        self.child
            .lock()
            .unwrap()
            .try_status()
            .ok()
            .flatten()
            .map(exit_code)
    }

    async fn wait(&mut self) -> Result<i32> {
        Ok(exit_code(self.child.get_mut().unwrap().status().await?))
    }

    async fn stop(&mut self) -> Result<()> {
        self.child.get_mut().unwrap().kill()?;
        Ok(())
    }
}

fn exit_code(c: ExitStatus) -> i32 {
    #[cfg(unix)]
    return c.code().or(c.signal()).unwrap();
    #[cfg(not(unix))]
    return c.code().unwrap();
}
