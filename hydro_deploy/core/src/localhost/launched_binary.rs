#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::sync::{Arc, Mutex};

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use futures::io::BufReader;
use futures::{AsyncBufReadExt, AsyncWriteExt, StreamExt};

use crate::util::prioritized_broadcast;
use crate::LaunchedBinary;

pub struct LaunchedLocalhostBinary {
    child: Mutex<async_process::Child>,
    stdin_sender: Sender<String>,
    stdout_cli_receivers: Arc<Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
    stdout_receivers: Arc<Mutex<Vec<Sender<String>>>>,
    stderr_receivers: Arc<Mutex<Vec<Sender<String>>>>,
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
        let (stdin_sender, mut stdin_receiver) = async_channel::unbounded::<String>();
        let mut stdin = child.stdin.take().unwrap();
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
    fn stdin(&self) -> Sender<String> {
        self.stdin_sender.clone()
    }

    fn cli_stdout(&self) -> tokio::sync::oneshot::Receiver<String> {
        let mut receivers = self.stdout_cli_receivers.lock().unwrap();

        if receivers.is_some() {
            panic!("Only one CLI stdout receiver is allowed at a time");
        }

        let (sender, receiver) = tokio::sync::oneshot::channel::<String>();
        *receivers = Some(sender);
        receiver
    }

    fn stdout(&self) -> Receiver<String> {
        let mut receivers = self.stdout_receivers.lock().unwrap();
        let (sender, receiver) = async_channel::unbounded::<String>();
        receivers.push(sender);
        receiver
    }

    fn stderr(&self) -> Receiver<String> {
        let mut receivers = self.stderr_receivers.lock().unwrap();
        let (sender, receiver) = async_channel::unbounded::<String>();
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
            .and_then(|c| {
                #[cfg(unix)]
                return c.code().or(c.signal());
                #[cfg(not(unix))]
                return c.code();
            })
    }

    async fn wait(&mut self) -> Option<i32> {
        let _ = self.child.get_mut().unwrap().status().await;
        self.exit_code()
    }
}
