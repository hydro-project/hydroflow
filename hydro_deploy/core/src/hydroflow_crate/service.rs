use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Result};
use async_channel::Receiver;
use async_trait::async_trait;
use futures_core::Future;
use hydroflow_cli_integration::{InitConfig, ServerPort};
use serde::Serialize;
use tokio::sync::RwLock;

use super::build::{build_crate_memoized, BuildError, BuildOutput, BuildParams};
use super::ports::{self, HydroflowPortConfig, HydroflowSink, SourcePath};
use crate::progress::ProgressTracker;
use crate::{
    Host, LaunchedBinary, LaunchedHost, ResourceBatch, ResourceResult, ServerStrategy, Service,
};

pub struct HydroflowCrateService {
    id: usize,
    pub(super) on: Arc<RwLock<dyn Host>>,
    build_params: BuildParams,
    perf: Option<PathBuf>,
    args: Option<Vec<String>>,
    display_id: Option<String>,
    external_ports: Vec<u16>,

    meta: Option<String>,

    /// Configuration for the ports this service will connect to as a client.
    pub(super) port_to_server: HashMap<String, ports::ServerConfig>,

    /// Configuration for the ports that this service will listen on a port for.
    pub(super) port_to_bind: HashMap<String, ServerStrategy>,

    launched_host: Option<Arc<dyn LaunchedHost>>,

    /// A map of port names to config for how other services can connect to this one.
    /// Only valid after `ready` has been called, only contains ports that are configured
    /// in `server_ports`.
    pub(super) server_defns: Arc<RwLock<HashMap<String, ServerPort>>>,

    launched_binary: Option<Arc<RwLock<dyn LaunchedBinary>>>,
    started: bool,
}

impl HydroflowCrateService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: usize,
        src: PathBuf,
        on: Arc<RwLock<dyn Host>>,
        bin: Option<String>,
        example: Option<String>,
        profile: Option<String>,
        perf: Option<PathBuf>,
        features: Option<Vec<String>>,
        args: Option<Vec<String>>,
        display_id: Option<String>,
        external_ports: Vec<u16>,
    ) -> Self {
        let target_type = on.try_read().unwrap().target_type();

        let build_params = BuildParams::new(src, bin, example, profile, target_type, features);

        Self {
            id,
            on,
            build_params,
            perf,
            args,
            display_id,
            external_ports,
            meta: None,
            port_to_server: HashMap::new(),
            port_to_bind: HashMap::new(),
            launched_host: None,
            server_defns: Arc::new(RwLock::new(HashMap::new())),
            launched_binary: None,
            started: false,
        }
    }

    pub fn update_meta<T: Serialize>(&mut self, meta: T) {
        if self.launched_binary.is_some() {
            panic!("Cannot update meta after binary has been launched")
        }

        self.meta = Some(serde_json::to_string(&meta).unwrap());
    }

    pub fn get_port(
        &self,
        name: String,
        self_arc: &Arc<RwLock<HydroflowCrateService>>,
    ) -> HydroflowPortConfig {
        HydroflowPortConfig {
            service: Arc::downgrade(self_arc),
            service_host: self.on.clone(),
            service_server_defns: self.server_defns.clone(),
            port: name,
            merge: false,
        }
    }

    pub fn add_connection(
        &mut self,
        self_arc: &Arc<RwLock<HydroflowCrateService>>,
        my_port: String,
        sink: &mut dyn HydroflowSink,
    ) -> Result<()> {
        let forward_res = sink.instantiate(&SourcePath::Direct(self.on.clone()));
        if let Ok(instantiated) = forward_res {
            // TODO(shadaj): if already in this map, we want to broadcast
            assert!(!self.port_to_server.contains_key(&my_port));
            self.port_to_server.insert(my_port, instantiated());
            Ok(())
        } else {
            drop(forward_res);
            let instantiated = sink.instantiate_reverse(
                &self.on,
                Arc::new(HydroflowPortConfig {
                    service: Arc::downgrade(self_arc),
                    service_host: self.on.clone(),
                    service_server_defns: self.server_defns.clone(),
                    port: my_port.clone(),
                    merge: false,
                }),
                &|p| p,
            )?;

            assert!(!self.port_to_bind.contains_key(&my_port));
            self.port_to_bind
                .insert(my_port, instantiated(sink.as_any_mut()));

            Ok(())
        }
    }

    pub async fn stdout(&self) -> Receiver<String> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .read()
            .await
            .stdout()
            .await
    }

    pub async fn stderr(&self) -> Receiver<String> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .read()
            .await
            .stderr()
            .await
    }

    pub async fn exit_code(&self) -> Option<i32> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .read()
            .await
            .exit_code()
            .await
    }

    fn build(&self) -> impl Future<Output = Result<&'static BuildOutput, BuildError>> {
        // Memoized, so no caching in `self` is needed.
        build_crate_memoized(self.build_params.clone())
    }
}

#[async_trait]
impl Service for HydroflowCrateService {
    fn collect_resources(&mut self, _resource_batch: &mut ResourceBatch) {
        if self.launched_host.is_some() {
            return;
        }

        tokio::task::spawn(self.build());

        let mut host = self
            .on
            .try_write()
            .expect("No one should be reading/writing the host while resources are collected");

        host.request_custom_binary();
        for (_, bind_type) in self.port_to_bind.iter() {
            host.request_port(bind_type);
        }

        for port in self.external_ports.iter() {
            host.request_port(&ServerStrategy::ExternalTcpPort(*port));
        }
    }

    async fn deploy(&mut self, resource_result: &Arc<ResourceResult>) -> Result<()> {
        if self.launched_host.is_some() {
            return Ok(());
        }

        ProgressTracker::with_group(
            &self
                .display_id
                .clone()
                .unwrap_or_else(|| format!("service/{}", self.id)),
            None,
            || async {
                let built = self.build().await?;

                let mut host_write = self.on.write().await;
                let launched = host_write.provision(resource_result).await;

                launched.copy_binary(built).await?;

                self.launched_host = Some(launched);
                Ok(())
            },
        )
        .await
    }

    async fn ready(&mut self) -> Result<()> {
        if self.launched_binary.is_some() {
            return Ok(());
        }

        ProgressTracker::with_group(
            &self
                .display_id
                .clone()
                .unwrap_or_else(|| format!("service/{}", self.id)),
            None,
            || async {
                let launched_host = self.launched_host.as_ref().unwrap();

                let built = self.build().await?;
                let args = self.args.as_ref().cloned().unwrap_or_default();

                let binary = launched_host
                    .launch_binary(
                        self.display_id
                            .clone()
                            .unwrap_or_else(|| format!("service/{}", self.id)),
                        built,
                        &args,
                        self.perf.clone(),
                    )
                    .await?;

                let mut bind_config = HashMap::new();
                for (port_name, bind_type) in self.port_to_bind.iter() {
                    bind_config.insert(port_name.clone(), launched_host.server_config(bind_type));
                }

                let formatted_bind_config =
                    serde_json::to_string::<InitConfig>(&(bind_config, self.meta.clone())).unwrap();

                // request stdout before sending config so we don't miss the "ready" response
                let stdout_receiver = binary.write().await.cli_stdout().await;

                binary
                    .write()
                    .await
                    .stdin()
                    .await
                    .send(format!("{formatted_bind_config}\n"))
                    .await?;

                let ready_line = ProgressTracker::leaf(
                    "waiting for ready".to_string(),
                    tokio::time::timeout(Duration::from_secs(60), stdout_receiver),
                )
                .await??;
                if ready_line.starts_with("ready: ") {
                    *self.server_defns.try_write().unwrap() =
                        serde_json::from_str(ready_line.trim_start_matches("ready: ")).unwrap();
                } else {
                    bail!("expected ready");
                }

                self.launched_binary = Some(binary);

                Ok(())
            },
        )
        .await
    }

    async fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }

        let mut sink_ports = HashMap::new();
        for (port_name, outgoing) in self.port_to_server.drain() {
            sink_ports.insert(port_name.clone(), outgoing.load_instantiated(&|p| p).await);
        }

        let formatted_defns = serde_json::to_string(&sink_ports).unwrap();

        let stdout_receiver = self
            .launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .cli_stdout()
            .await;

        self.launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .stdin()
            .await
            .send(format!("start: {formatted_defns}\n"))
            .await
            .unwrap();

        let start_ack_line = ProgressTracker::leaf(
            "waiting for ack start".to_string(),
            tokio::time::timeout(Duration::from_secs(60), stdout_receiver),
        )
        .await??;
        if !start_ack_line.starts_with("ack start") {
            bail!("expected ack start");
        }

        self.started = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .stdin()
            .await
            .send("stop\n".to_string())
            .await?;

        self.launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .wait()
            .await;

        Ok(())
    }
}
