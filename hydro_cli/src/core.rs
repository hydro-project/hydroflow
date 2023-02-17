use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_channel::Receiver;
use async_process::{Command, Stdio};
use async_trait::async_trait;
use cargo::ops::CompileFilter;
use futures_lite::{io::BufReader, prelude::*};
use tokio::{sync::RwLock, task::JoinHandle};

#[derive(Clone)]
pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
    pub services: Vec<Arc<RwLock<dyn Service>>>,
}

impl Deployment {
    pub async fn deploy(&mut self) {
        let hosts_future = self
            .hosts
            .iter_mut()
            .map(|host: &mut Arc<RwLock<dyn Host>>| async {
                host.write().await.provision().await;
            });

        let all_hosts = futures::future::join_all(hosts_future);

        let services_future =
            self.services
                .iter_mut()
                .map(|service: &mut Arc<RwLock<dyn Service>>| async {
                    service.write().await.deploy().await;
                });

        let all_services = futures::future::join_all(services_future);

        futures::future::join(all_hosts, all_services).await;
    }

    pub fn add_host<T: Host + 'static>(&mut self, host: T) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(host));
        self.hosts.push(arc.clone());
        arc
    }

    pub fn add_service<T: Service + 'static>(&mut self, service: T) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(service));
        self.services.push(arc.clone());
        arc
    }
}

impl Debug for Deployment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deployment")
            .field(
                "hosts",
                &self
                    .hosts
                    .iter()
                    .map(|h| h.blocking_read())
                    .collect::<Vec<_>>(),
            )
            .field(
                "services",
                &self
                    .services
                    .iter()
                    .map(|h| h.blocking_read())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[async_trait]
pub trait LaunchedBinary: Send + Sync {
    fn lines(&self) -> Receiver<String>;
}

struct LaunchedLocalhostBinary {
    _child: async_process::Child,
    stdout_channel: Receiver<String>,
}

#[async_trait]
impl LaunchedBinary for LaunchedLocalhostBinary {
    fn lines(&self) -> Receiver<String> {
        self.stdout_channel.clone()
    }
}

#[async_trait]
pub trait LaunchedHost: Send + Sync {
    async fn launch_binary(&self, binary: String) -> Arc<RwLock<dyn LaunchedBinary>>;
}

struct LaunchedLocalhost {}

#[async_trait]
impl LaunchedHost for LaunchedLocalhost {
    async fn launch_binary(&self, binary: String) -> Arc<RwLock<dyn LaunchedBinary>> {
        let mut child = Command::new(binary).stdout(Stdio::piped()).spawn().unwrap();

        let (sender, receiver) = async_channel::unbounded();

        let stdout = child.stdout.take().unwrap();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();

            while let Some(line) = lines.next().await {
                if sender.send(line.unwrap()).await.is_err() {
                    break;
                }
            }
        });

        Arc::new(RwLock::new(LaunchedLocalhostBinary {
            _child: child,
            stdout_channel: receiver,
        }))
    }
}

#[async_trait]
pub trait Host: Send + Sync + Debug {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost>;
}

#[derive(Debug)]
pub struct LocalhostHost {}

#[async_trait]
impl Host for LocalhostHost {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {})
    }
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
    async fn deploy(&mut self);

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct HydroflowCrate {
    pub src: PathBuf,
    pub on: Arc<RwLock<dyn Host>>,
    pub example: Option<String>,

    // actually HydroflowCrates
    pub outgoing_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,
    pub incoming_ports: HashMap<String, (Weak<RwLock<dyn Service>>, String)>,

    pub launched_binary: Option<Arc<RwLock<dyn LaunchedBinary>>>,
}

impl HydroflowCrate {
    pub fn stdout(&self) -> Receiver<String> {
        self.launched_binary
            .as_ref()
            .unwrap()
            .blocking_read()
            .lines()
    }

    fn build(&mut self) -> JoinHandle<String> {
        let src_cloned = self.src.join("Cargo.toml").canonicalize().unwrap();
        let example_cloned = self.example.clone();

        tokio::task::spawn_blocking(move || {
            let config = cargo::Config::default().unwrap();
            let workspace = cargo::core::Workspace::new(&src_cloned, &config).unwrap();

            let mut compile_options =
                cargo::ops::CompileOptions::new(&config, cargo::core::compiler::CompileMode::Build)
                    .unwrap();
            compile_options.filter = CompileFilter::Only {
                all_targets: false,
                lib: cargo::ops::LibRule::Default,
                bins: cargo::ops::FilterRule::Just(vec![]),
                examples: cargo::ops::FilterRule::Just(vec![example_cloned.unwrap()]),
                tests: cargo::ops::FilterRule::Just(vec![]),
                benches: cargo::ops::FilterRule::Just(vec![]),
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
    async fn deploy(&mut self) {
        let built = self.build();
        let mut host = self.on.write().await;
        let launched = host.provision();

        self.launched_binary = Some(launched.await.launch_binary(built.await.unwrap()).await);
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
