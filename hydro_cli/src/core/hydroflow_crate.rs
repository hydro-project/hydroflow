use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_channel::Receiver;
use async_trait::async_trait;
use cargo::{
    core::Workspace,
    ops::{CompileFilter, CompileOptions, FilterRule, LibRule},
    util::command_prelude::CompileMode,
    Config,
};
use hydroflow::util::connection::ConnectionPipe;
use tokio::{sync::RwLock, task::JoinHandle};

use super::{Host, LaunchedBinary, LaunchedHost, Service, TerraformBatch, TerraformResult};

pub struct HydroflowCrate {
    pub src: PathBuf,
    pub on: Arc<RwLock<dyn Host>>,
    pub example: Option<String>,

    // actually HydroflowCrates
    pub outgoing_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,
    pub incoming_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,

    pub built_binary: Option<String>,
    pub launched_host: Option<Arc<dyn LaunchedHost>>,
    pub allocated_incoming: HashMap<String, (ConnectionPipe, Box<dyn Any + Send + Sync>)>,
    pub launched_binary: Option<Arc<RwLock<dyn LaunchedBinary>>>,
}

impl HydroflowCrate {
    pub fn new(src: PathBuf, on: Arc<RwLock<dyn Host>>, example: Option<String>) -> Self {
        Self {
            src,
            on,
            example,
            outgoing_ports: HashMap::new(),
            incoming_ports: HashMap::new(),
            built_binary: None,
            launched_host: None,
            allocated_incoming: HashMap::new(),
            launched_binary: None,
        }
    }

    pub fn stdout(&self) -> Receiver<String> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .blocking_read()
            .stdout()
    }

    pub fn stderr(&self) -> Receiver<String> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .blocking_read()
            .stderr()
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .blocking_read()
            .exit_code()
    }

    fn build(&mut self) -> JoinHandle<String> {
        let src_cloned = self.src.join("Cargo.toml").canonicalize().unwrap();
        let example_cloned = self.example.clone();

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
    async fn collect_resources(&mut self, terraform: &mut TerraformBatch) {
        let mut host = self.on.write().await;
        host.collect_resources(terraform).await;
    }

    async fn deploy(&mut self, terraform_result: &TerraformResult) {
        let built = self.build();
        let host_read = self.on.read().await;

        self.allocated_incoming = HashMap::new();
        for (port_name, (service, _)) in self.incoming_ports.iter() {
            let service = service.upgrade().unwrap();
            let pipe = host_read.allocate_pipe(service.read().await.host()).await;
            self.allocated_incoming.insert(port_name.clone(), pipe);
        }

        drop(host_read);

        let mut host_write = self.on.write().await;
        let launched = host_write.provision(terraform_result);

        self.built_binary = Some(built.await.unwrap());
        self.launched_host = Some(launched.await);
    }

    async fn ready(&mut self) {
        let binary = self
            .launched_host
            .as_ref()
            .unwrap()
            .launch_binary(self.built_binary.as_ref().unwrap().clone())
            .await;

        let mut port_config = HashMap::new();
        for (port_name, (conn_type, _)) in self.allocated_incoming.iter() {
            port_config.insert(port_name.clone(), conn_type.clone());
        }

        for (port_name, (target, target_port)) in self.outgoing_ports.iter() {
            let target = target.upgrade().unwrap();
            let target = target.read().await;
            let target = target.as_any().downcast_ref::<HydroflowCrate>().unwrap();

            let (conn_type, _) = target.allocated_incoming.get(target_port).unwrap();

            port_config.insert(port_name.clone(), conn_type.clone());
        }

        let formatted_ports = serde_json::to_string(&port_config).unwrap();

        binary
            .write()
            .await
            .stdin()
            .send(format!("{formatted_ports}\n"))
            .await
            .unwrap();

        if binary.write().await.stdout().recv().await.unwrap() != "ready" {
            panic!("expected ready");
        }

        self.launched_binary = Some(binary);
    }

    async fn start(&mut self) {
        self.launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .stdin()
            .send("start\n".to_string())
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
