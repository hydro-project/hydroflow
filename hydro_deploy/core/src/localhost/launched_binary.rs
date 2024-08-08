#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use futures::io::BufReader as FuturesBufReader;
use futures::{AsyncBufReadExt, AsyncWriteExt};
use inferno::collapse::dtrace::Folder;
use inferno::collapse::Collapse;
use tokio::sync::{mpsc, oneshot};

use crate::hydroflow_crate::flamegraph::handle_fold_data;
use crate::hydroflow_crate::tracing_options::TracingOptions;
use crate::progress::ProgressTracker;
use crate::util::prioritized_broadcast;
use crate::LaunchedBinary;

pub struct LaunchedLocalhostBinary {
    child: Mutex<async_process::Child>,
    tracing: Option<TracingOptions>,
    stdin_sender: mpsc::UnboundedSender<String>,
    stdout_deploy_receivers: Arc<Mutex<Option<oneshot::Sender<String>>>>,
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
            ProgressTracker::println(format!("Failed to SIGTERM process {}: {}", pid, e));
        }
    }
}

impl LaunchedLocalhostBinary {
    pub fn new(
        mut child: async_process::Child,
        id: String,
        tracing: Option<TracingOptions>,
    ) -> Self {
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
        let (stdout_deploy_receivers, stdout_receivers) = prioritized_broadcast(
            FuturesBufReader::new(child.stdout.take().unwrap()).lines(),
            move |s| ProgressTracker::println(format!("[{id_clone}] {s}")),
        );
        let (_, stderr_receivers) = prioritized_broadcast(
            FuturesBufReader::new(child.stderr.take().unwrap()).lines(),
            move |s| ProgressTracker::println(&format!("[{id} stderr] {s}")),
        );

        Self {
            child: Mutex::new(child),
            tracing,
            stdin_sender,
            stdout_deploy_receivers,
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
        if let Err(err) = self.child.get_mut().unwrap().kill() {
            if !matches!(err.kind(), std::io::ErrorKind::InvalidInput) {
                Err(err)?;
            }
        }

        // Run perf post-processing and download perf output.
        if let Some(tracing) = self.tracing.as_ref() {
            let dtrace_outfile = tracing
                .dtrace_outfile
                .as_ref()
                .expect("`dtrace_outfile` must be set for `dtrace` on localhost.");
            let mut fold_er = Folder::from(tracing.fold_dtrace_options.clone().unwrap_or_default());

            let fold_data = ProgressTracker::leaf("fold dtrace output".to_owned(), async move {
                let mut fold_data = Vec::new();
                fold_er.collapse_file(Some(dtrace_outfile), &mut fold_data)?;
                Result::<_>::Ok(fold_data)
            })
            .await?;

            handle_fold_data(tracing, fold_data).await?;
        };

        Ok(())
    }
}

fn exit_code(c: ExitStatus) -> i32 {
    #[cfg(unix)]
    return c.code().or(c.signal()).unwrap();
    #[cfg(not(unix))]
    return c.code().unwrap();
}
