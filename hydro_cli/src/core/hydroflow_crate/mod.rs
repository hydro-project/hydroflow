use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{bail, Result};
use async_channel::Receiver;
use async_trait::async_trait;
use cargo::{
    core::{compiler::BuildConfig, resolver::CliFeatures, Workspace},
    ops::{CompileFilter, CompileOptions, FilterRule, LibRule},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};
use hydroflow_cli_integration::ServerPort;
use tokio::{sync::RwLock, task::JoinHandle};

use self::ports::HydroflowPortConfig;

use super::{
    Host, HostTargetType, LaunchedBinary, LaunchedHost, ResourceBatch, ResourceResult,
    ServerStrategy, Service,
};

pub mod ports;

pub struct HydroflowCrate {
    src: PathBuf,
    on: Arc<RwLock<dyn Host>>,
    example: Option<String>,
    features: Option<Vec<String>>,
    args: Option<Vec<String>>,

    /// Configuration for the ports this service will connect to as a client.
    client_port: HashMap<String, ports::ClientPort>,

    /// Configuration for the ports that this service will listen on a port for.
    server_ports: HashMap<String, ServerStrategy>,

    built_binary: Option<JoinHandle<PathBuf>>,
    launched_host: Option<Arc<dyn LaunchedHost>>,

    /// A map of port names to config for how other services can connect to this one.
    /// Only valid after `ready` has been called, only contains ports that are configured
    /// in `server_ports`.
    server_defns: HashMap<String, ServerPort>,

    launched_binary: Option<Arc<RwLock<dyn LaunchedBinary>>>,
}

impl HydroflowCrate {
    pub fn new(
        src: PathBuf,
        on: Arc<RwLock<dyn Host>>,
        example: Option<String>,
        features: Option<Vec<String>>,
        args: Option<Vec<String>>,
    ) -> Self {
        Self {
            src,
            on,
            example,
            features,
            args,
            client_port: HashMap::new(),
            server_ports: HashMap::new(),
            built_binary: None,
            launched_host: None,
            server_defns: HashMap::new(),
            launched_binary: None,
        }
    }

    pub fn add_client_port(
        &mut self,
        self_arc: &Arc<RwLock<HydroflowCrate>>,
        my_port: String,
        config: ports::HydroflowPortConfig,
    ) -> Result<()> {
        if let Ok(instantiated) = config.instantiate(&self.on) {
            self.client_port.insert(my_port, instantiated);
            Ok(())
        } else {
            let instantiated = config.instantiate_reverse(
                &self.on,
                Box::new(HydroflowPortConfig::Direct(
                    Arc::downgrade(self_arc),
                    my_port.clone(),
                )),
                &|p| p,
            )?;
            self.server_ports.insert(my_port, instantiated);
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

    fn build(&mut self) -> JoinHandle<PathBuf> {
        let src_cloned = self.src.join("Cargo.toml").canonicalize().unwrap();
        let example_cloned = self.example.clone();
        let features_cloned = self.features.clone();
        let host = self.on.clone();

        tokio::task::spawn_blocking(move || {
            let config = Config::default().unwrap();
            config.shell().set_verbosity(cargo::core::Verbosity::Normal);

            let workspace = Workspace::new(&src_cloned, &config).unwrap();

            let mut compile_options = CompileOptions::new(&config, CompileMode::Build).unwrap();
            compile_options.filter = CompileFilter::Only {
                all_targets: false,
                lib: LibRule::Default,
                bins: FilterRule::Just(vec![]),
                examples: FilterRule::Just(vec![example_cloned.unwrap()]),
                tests: FilterRule::Just(vec![]),
                benches: FilterRule::Just(vec![]),
            };

            let target_type = host.blocking_read().target_type();

            compile_options.build_config = BuildConfig::new(
                &config,
                None,
                false,
                &(match target_type {
                    HostTargetType::Local => vec![],
                    HostTargetType::Linux => vec!["x86_64-unknown-linux-musl".to_string()],
                }),
                CompileMode::Build,
            )
            .unwrap();

            compile_options.build_config.requested_profile = InternedString::from("release");
            compile_options.cli_features =
                CliFeatures::from_command_line(&features_cloned.unwrap_or_default(), false, true)
                    .unwrap();

            let res = cargo::ops::compile(&workspace, &compile_options).unwrap();
            let binaries = res
                .binaries
                .iter()
                .map(|b| b.path.to_string_lossy())
                .collect::<Vec<_>>();

            if binaries.len() == 1 {
                binaries[0].to_string().into()
            } else {
                panic!("expected exactly one binary, got {}", binaries.len())
            }
        })
    }
}

#[async_trait]
impl Service for HydroflowCrate {
    fn collect_resources(&mut self, _resource_batch: &mut ResourceBatch) {
        let built = self.build();
        self.built_binary = Some(built);

        let mut host = self
            .on
            .try_write()
            .expect("No one should be reading/writing the host while resources are collected");

        host.request_custom_binary();
        for (_, bind_type) in self.server_ports.iter() {
            host.request_port(bind_type);
        }
    }

    async fn deploy(&mut self, resource_result: &Arc<ResourceResult>) {
        let mut host_write = self.on.write().await;
        let launched = host_write.provision(resource_result);
        self.launched_host = Some(launched.await);
    }

    async fn ready(&mut self) -> Result<()> {
        let launched_host = self.launched_host.as_ref().unwrap();

        let binary = launched_host
            .launch_binary(
                self.built_binary.take().unwrap().await.as_ref().unwrap(),
                &self.args.as_ref().cloned().unwrap_or_default(),
            )
            .await?;

        let mut bind_config = HashMap::new();
        for (port_name, bind_type) in self.server_ports.iter() {
            bind_config.insert(port_name.clone(), launched_host.server_config(bind_type));
        }

        let formatted_bind_config = serde_json::to_string(&bind_config).unwrap();

        // request stdout before sending config so we don't miss the "ready" response
        let stdout_receiver = binary.write().await.stdout().await;

        binary
            .write()
            .await
            .stdin()
            .await
            .send(format!("{formatted_bind_config}\n"))
            .await?;

        let ready_line = stdout_receiver.recv().await.unwrap();
        if ready_line.starts_with("ready: ") {
            self.server_defns =
                serde_json::from_str(ready_line.trim_start_matches("ready: ")).unwrap();
        } else {
            bail!("expected ready");
        }

        self.launched_binary = Some(binary);

        Ok(())
    }

    async fn start(&mut self) {
        let mut connection_defns = HashMap::new();
        for (port_name, outgoing) in self.client_port.drain() {
            connection_defns.insert(port_name.clone(), outgoing.connection_defn().await);
        }

        let formatted_defns = serde_json::to_string(&connection_defns).unwrap();

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
    }
}
