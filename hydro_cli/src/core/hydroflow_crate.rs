use std::{
    collections::HashMap,
    fmt::Debug,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_channel::Receiver;
use async_trait::async_trait;
use cargo::{
    core::{compiler::BuildConfig, resolver::CliFeatures, Workspace},
    ops::{CompileFilter, CompileOptions, FilterRule, LibRule},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};
use hydroflow::util::connection::ConnectionPipe;
use tokio::{sync::RwLock, task::JoinHandle};

use super::{Host, LaunchedBinary, LaunchedHost, ResourceBatch, ResourceResult, Service};

pub struct HydroflowCrate {
    src: PathBuf,
    on: Arc<RwLock<dyn Host>>,
    example: Option<String>,
    features: Option<Vec<String>>,

    /// A mapping from output ports to the service that the port sends data to.
    /// Each `dyn Service` is guaranteed to be another `HydroflowCrate`.
    outgoing_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,

    /// A map of port names to the host that will be sending data to the port.
    incoming_ports: HashMap<String, Arc<RwLock<dyn Host>>>,

    built_binary: Option<String>,
    launched_host: Option<Arc<dyn LaunchedHost>>,

    /// A map of port names to config for how other services can connect to this one.
    /// Only valid after `ready` has been called.
    bound_pipes: HashMap<String, ConnectionPipe>,

    launched_binary: Option<Arc<RwLock<dyn LaunchedBinary>>>,
}

impl HydroflowCrate {
    pub fn new(
        src: PathBuf,
        on: Arc<RwLock<dyn Host>>,
        example: Option<String>,
        features: Option<Vec<String>>,
    ) -> Self {
        Self {
            src,
            on,
            example,
            features,
            outgoing_ports: HashMap::new(),
            incoming_ports: HashMap::new(),
            built_binary: None,
            launched_host: None,
            bound_pipes: HashMap::new(),
            launched_binary: None,
        }
    }

    pub fn add_incoming_port(&mut self, my_port: String, from: &HydroflowCrate) {
        self.incoming_ports.insert(my_port, from.host());
    }

    pub fn add_outgoing_port(
        &mut self,
        my_port: String,
        to: &Arc<RwLock<dyn Service>>,
        to_port: String,
    ) {
        self.outgoing_ports
            .insert(my_port, (Arc::downgrade(to), to_port));
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

    fn build(&mut self) -> JoinHandle<String> {
        let src_cloned = self.src.join("Cargo.toml").canonicalize().unwrap();
        let example_cloned = self.example.clone();
        let features_cloned = self.features.clone();

        tokio::task::spawn_blocking(move || {
            let config = Config::default().unwrap();
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

            compile_options.build_config = BuildConfig::new(
                &config,
                None,
                false,
                &[
                    // TODO(shadaj): make configurable
                    "x86_64-unknown-linux-musl".to_string(),
                ],
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
                binaries[0].to_string()
            } else {
                panic!("expected exactly one binary, got {}", binaries.len())
            }
        })
    }
}

#[async_trait]
impl Service for HydroflowCrate {
    async fn collect_resources(&mut self, resource_batch: &mut ResourceBatch) {
        let built = self.build();
        self.built_binary = Some(built.await.unwrap());
        let host_read = self.on.read().await;

        drop(host_read);

        let mut host = self.on.write().await;
        host.collect_resources(resource_batch).await;
    }

    async fn deploy(&mut self, resource_result: &Arc<ResourceResult>) {
        let mut host_write = self.on.write().await;
        let launched = host_write.provision(resource_result);
        self.launched_host = Some(launched.await);
    }

    async fn ready(&mut self) {
        let binary = self
            .launched_host
            .as_ref()
            .unwrap()
            .launch_binary(self.built_binary.as_ref().unwrap().clone())
            .await;

        let mut bind_types = HashMap::new();
        for (port_name, from_host) in self.incoming_ports.iter() {
            bind_types.insert(
                port_name.clone(),
                from_host
                    .read()
                    .await
                    .find_bind_type(self.on.read().await.deref()),
            );
        }

        let formatted_bind_types = serde_json::to_string(&bind_types).unwrap();

        // request stdout before sending config so we don't miss the "ready" response
        let stdout_receiver = binary.write().await.stdout().await;

        binary
            .write()
            .await
            .stdin()
            .await
            .send(format!("{formatted_bind_types}\n"))
            .await
            .unwrap();

        let ready_line = stdout_receiver.recv().await.unwrap();
        if ready_line.starts_with("ready: ") {
            self.bound_pipes =
                serde_json::from_str(ready_line.trim_start_matches("ready: ")).unwrap();
        } else {
            panic!("expected ready");
        }

        self.launched_binary = Some(binary);
    }

    async fn start(&mut self) {
        let mut pipe_config = HashMap::new();
        for (port_name, (target, target_port)) in self.outgoing_ports.iter() {
            let target = target.upgrade().unwrap();
            let target = target.read().await;
            let target = target.as_any().downcast_ref::<HydroflowCrate>().unwrap();

            let conn = target.bound_pipes.get(target_port).unwrap();

            pipe_config.insert(port_name.clone(), conn.clone());
        }

        let formatted_pipe_config = serde_json::to_string(&pipe_config).unwrap();

        self.launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .stdin()
            .await
            .send(format!("start: {formatted_pipe_config}\n"))
            .await
            .unwrap();
    }

    fn host(&self) -> Arc<RwLock<dyn Host>> {
        self.on.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Debug for HydroflowCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HydroflowCrate")
            .field("src", &self.src)
            .field("on", &self.on.blocking_read())
            .field("example", &self.example)
            .field("outgoing_ports", &self.outgoing_ports)
            .field("incoming_ports", &self.incoming_ports)
            .finish()
    }
}
