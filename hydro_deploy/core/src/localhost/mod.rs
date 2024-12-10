use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use async_process::{Command, Stdio};
use async_trait::async_trait;
use hydroflow_deploy_integration::ServerBindConfig;
use nameof::name_of;

use super::{
    ClientStrategy, Host, HostTargetType, LaunchedBinary, LaunchedHost, ResourceBatch,
    ResourceResult, ServerStrategy,
};
use crate::hydroflow_crate::build::BuildOutput;
use crate::hydroflow_crate::tracing_options::TracingOptions;
use crate::progress::ProgressTracker;
use crate::HostStrategyGetter;

pub mod launched_binary;
pub use launched_binary::*;

#[derive(Debug)]
pub struct LocalhostHost {
    pub id: usize,
    client_only: bool,
}

impl LocalhostHost {
    pub fn new(id: usize) -> LocalhostHost {
        LocalhostHost {
            id,
            client_only: false,
        }
    }

    pub fn client_only(&self) -> LocalhostHost {
        LocalhostHost {
            id: self.id,
            client_only: true,
        }
    }
}

#[async_trait]
impl Host for LocalhostHost {
    fn target_type(&self) -> HostTargetType {
        HostTargetType::Local
    }

    fn request_port(&self, _bind_type: &ServerStrategy) {}
    fn collect_resources(&self, _resource_batch: &mut ResourceBatch) {}
    fn request_custom_binary(&self) {}

    fn id(&self) -> usize {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn launched(&self) -> Option<Arc<dyn LaunchedHost>> {
        Some(Arc::new(LaunchedLocalhost))
    }

    async fn provision(&self, _resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost)
    }

    fn strategy_as_server<'a>(
        &'a self,
        connection_from: &dyn Host,
    ) -> Result<(ClientStrategy<'a>, HostStrategyGetter)> {
        if self.client_only {
            anyhow::bail!("Localhost cannot be a server if it is client only")
        }

        if connection_from.can_connect_to(ClientStrategy::UnixSocket(self.id)) {
            Ok((
                ClientStrategy::UnixSocket(self.id),
                Box::new(|_| ServerStrategy::UnixSocket),
            ))
        } else if connection_from.can_connect_to(ClientStrategy::InternalTcpPort(self)) {
            Ok((
                ClientStrategy::InternalTcpPort(self),
                Box::new(|_| ServerStrategy::InternalTcpPort),
            ))
        } else {
            anyhow::bail!("Could not find a strategy to connect to localhost")
        }
    }

    fn can_connect_to(&self, typ: ClientStrategy) -> bool {
        match typ {
            ClientStrategy::UnixSocket(id) => {
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
            ClientStrategy::InternalTcpPort(target_host) => self.id == target_host.id(),
            ClientStrategy::ForwardedTcpPort(_) => true,
        }
    }
}

struct LaunchedLocalhost;

#[async_trait]
impl LaunchedHost for LaunchedLocalhost {
    fn server_config(&self, bind_type: &ServerStrategy) -> ServerBindConfig {
        match bind_type {
            ServerStrategy::UnixSocket => ServerBindConfig::UnixSocket,
            ServerStrategy::InternalTcpPort => ServerBindConfig::TcpPort("127.0.0.1".to_string()),
            ServerStrategy::ExternalTcpPort(_) => panic!("Cannot bind to external port"),
            ServerStrategy::Demux(demux) => {
                let mut config_map = HashMap::new();
                for (key, underlying) in demux {
                    config_map.insert(*key, self.server_config(underlying));
                }

                ServerBindConfig::Demux(config_map)
            }
            ServerStrategy::Merge(merge) => {
                let mut configs = vec![];
                for underlying in merge {
                    configs.push(self.server_config(underlying));
                }

                ServerBindConfig::Merge(configs)
            }
            ServerStrategy::Tagged(underlying, id) => {
                ServerBindConfig::Tagged(Box::new(self.server_config(underlying)), *id)
            }
            ServerStrategy::Null => ServerBindConfig::Null,
        }
    }

    async fn copy_binary(&self, _binary: &BuildOutput) -> Result<()> {
        Ok(())
    }

    async fn launch_binary(
        &self,
        id: String,
        binary: &BuildOutput,
        args: &[String],
        tracing: Option<TracingOptions>,
    ) -> Result<Box<dyn LaunchedBinary>> {
        let mut command = if let Some(tracing) = tracing.as_ref() {
            if cfg!(target_os = "macos") || cfg!(target_family = "windows") {
                // dtrace
                ProgressTracker::println(
                    format!("[{id} tracing] Profiling binary with `dtrace`.",),
                );
                let dtrace_outfile = tracing.dtrace_outfile.as_ref().ok_or_else(|| {
                    anyhow!(
                        "`{}` must be set for `dtrace` on localhost.",
                        name_of!(dtrace_outfile in TracingOptions)
                    )
                })?;

                let mut command = Command::new("dtrace");
                command
                    .arg("-o")
                    .arg(dtrace_outfile)
                    .arg("-n")
                    .arg(format!(
                        "profile-{} /pid == $target/ {{ @[ustack()] = count(); }}",
                        tracing.frequency
                    ))
                    .arg("-c")
                    .arg({
                        // TODO(mingwei): use std `intersperse` when stabilized.
                        let inner_command = itertools::Itertools::intersperse(
                            std::iter::once(binary.bin_path.to_str().unwrap())
                                .chain(args.iter().map(Deref::deref))
                                .map(|s| shell_escape::unix::escape(s.into())),
                            Cow::Borrowed(" "),
                        )
                        .collect::<String>();
                        &*shell_escape::unix::escape(inner_command.into())
                    });
                command
            }
            // else if cfg!(target_family = "windows") {
            //     // blondie_dtrace
            //     ProgressTracker::println(&format!(
            //         "[{id} tracing] Profiling binary with `blondie`. `TracingOptions::frequency` is ignored. Ensure that this is run as admin.",
            //     ));
            //     ProgressTracker::println(&format!(
            //         "[{id} tracing] Install `blondie` via `cargo install blondie --all-features`.",
            //     ));
            //     let _ = tracing;
            //     let mut command = Command::new("blondie");
            //     command
            //         .arg("-o")
            //         .arg(format!(
            //             "./blondie-{}.stacks",
            //             nanoid::nanoid!(5), // TODO!
            //         ))
            //         .arg("folded-text")
            //         .arg(&binary.bin_path)
            //         .args(args);
            //     command
            // }
            else if cfg!(target_family = "unix") {
                // perf
                ProgressTracker::println(format!("[{} tracing] Tracing binary with `perf`.", id));
                let perf_outfile = tracing.perf_raw_outfile.as_ref().ok_or_else(|| {
                    anyhow!(
                        "`{}` must be set for `perf` on localhost.",
                        name_of!(perf_raw_outfile in TracingOptions)
                    )
                })?;

                let mut command = Command::new("perf");
                command
                    .args([
                        "record",
                        "-F",
                        &tracing.frequency.to_string(),
                        "-e",
                        "cycles:u",
                        "--call-graph",
                        "dwarf,65528",
                        "-o",
                    ])
                    .arg(perf_outfile)
                    .arg(&binary.bin_path)
                    .args(args);
                command
            } else {
                bail!(
                    "Unknown OS for perf/dtrace tracing: {}",
                    std::env::consts::OS
                );
            }
        } else {
            let mut command = Command::new(&binary.bin_path);
            command.args(args);
            command
        };

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(not(target_family = "unix"))]
        command.kill_on_drop(true);

        ProgressTracker::println(format!("[{}] running command: `{:?}`", id, command));

        let child = command
            .spawn()
            .with_context(|| format!("Failed to execute command: {:?}", command))?;

        Ok(Box::new(LaunchedLocalhostBinary::new(child, id, tracing)))
    }

    async fn forward_port(&self, addr: &SocketAddr) -> Result<SocketAddr> {
        Ok(*addr)
    }
}
