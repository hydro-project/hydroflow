use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_channel::{Receiver, Sender};
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

        let all_services_ready =
            self.services
                .iter()
                .map(|service: &Arc<RwLock<dyn Service>>| async {
                    service.write().await.ready().await;
                });

        futures::future::join_all(all_services_ready).await;

        let all_services_start =
            self.services
                .iter()
                .map(|service: &Arc<RwLock<dyn Service>>| async {
                    service.write().await.start().await;
                });

        futures::future::join_all(all_services_start).await;
    }

    pub fn add_host<T: Host + 'static, F: Fn(usize) -> T>(&mut self, host: F) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(host(self.hosts.len())));
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
    fn stdin(&self) -> Sender<String>;
    fn stdout(&self) -> Receiver<String>;
    fn stderr(&self) -> Receiver<String>;

    fn exit_code(&self) -> Option<i32>;
}

struct LaunchedLocalhostBinary {
    child: RwLock<async_process::Child>,
    stdin_channel: Sender<String>,
    stdout_channel: Receiver<String>,
    stderr_channel: Receiver<String>,
}

#[async_trait]
impl LaunchedBinary for LaunchedLocalhostBinary {
    fn stdin(&self) -> Sender<String> {
        self.stdin_channel.clone()
    }

    fn stdout(&self) -> Receiver<String> {
        self.stdout_channel.clone()
    }

    fn stderr(&self) -> Receiver<String> {
        self.stderr_channel.clone()
    }

    fn exit_code(&self) -> Option<i32> {
        self.child
            .blocking_write()
            .try_status()
            .map(|s| s.and_then(|c| c.code()))
            .unwrap_or(None)
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
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let (stdin_sender, mut stdin_receiver) = async_channel::unbounded::<String>();
        let mut stdin = child.stdin.take().unwrap();
        tokio::spawn(async move {
            while let Some(line) = stdin_receiver.next().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let (stdout_sender, stdout_receiver) = async_channel::unbounded();
        let stdout = child.stdout.take().unwrap();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();

            while let Some(line) = lines.next().await {
                if stdout_sender.send(line.unwrap()).await.is_err() {
                    break;
                }
            }
        });

        let (stderr_sender, stderr_receiver) = async_channel::unbounded();
        let stderr = child.stderr.take().unwrap();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();

            while let Some(line) = lines.next().await {
                if stderr_sender.send(line.unwrap()).await.is_err() {
                    break;
                }
            }
        });

        Arc::new(RwLock::new(LaunchedLocalhostBinary {
            child: RwLock::new(child),
            stdin_channel: stdin_sender,
            stdout_channel: stdout_receiver,
            stderr_channel: stderr_receiver,
        }))
    }
}

#[derive(Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
}

pub enum ConnectionType {
    UnixSocket(usize),
}

#[async_trait]
pub trait Host: Send + Sync + Debug {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost>;

    async fn allocate_pipe(&self, client: Arc<RwLock<dyn Host>>) -> ConnectionPipe;

    fn can_connect_to(&self, typ: ConnectionType) -> bool;
}

#[derive(Debug)]
pub struct LocalhostHost {
    pub id: usize,
}

#[async_trait]
impl Host for LocalhostHost {
    async fn provision(&mut self) -> Arc<dyn LaunchedHost> {
        Arc::new(LaunchedLocalhost {})
    }

    async fn allocate_pipe(&self, client: Arc<RwLock<dyn Host>>) -> ConnectionPipe {
        if client
            .read()
            .await
            .can_connect_to(ConnectionType::UnixSocket(self.id))
        {
            // TODO(allocate a random file)
            ConnectionPipe::UnixSocket(PathBuf::from("/tmp/hydroflow.sock"))
        } else {
            todo!()
        }
    }

    fn can_connect_to(&self, typ: ConnectionType) -> bool {
        match typ {
            ConnectionType::UnixSocket(id) => self.id == id,
        }
    }
}

#[async_trait]
pub trait Service: Send + Sync + Debug {
    async fn deploy(&mut self);
    async fn ready(&mut self);
    async fn start(&mut self);

    fn host(&self) -> Arc<RwLock<dyn Host>>;

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

    pub built_binary: Option<String>,
    pub launched_host: Option<Arc<dyn LaunchedHost>>,
    pub allocated_incoming: HashMap<String, ConnectionPipe>,
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
        let host_read = self.on.read().await;

        self.allocated_incoming = HashMap::new();
        for (port_name, (service, _)) in self.incoming_ports.iter() {
            let service = service.upgrade().unwrap();
            let pipe = host_read.allocate_pipe(service.read().await.host()).await;
            self.allocated_incoming.insert(port_name.clone(), pipe);
        }

        drop(host_read);

        let mut host_write = self.on.write().await;
        let launched = host_write.provision();

        self.built_binary = Some(built.await.unwrap());
        self.launched_host = Some(launched.await);
    }

    async fn ready(&mut self) {
        self.launched_binary = Some(
            self.launched_host
                .as_ref()
                .unwrap()
                .launch_binary(self.built_binary.as_ref().unwrap().clone())
                .await,
        );
    }

    async fn start(&mut self) {
        let mut all_ports = self.allocated_incoming.clone();
        for (port_name, (target, target_port)) in self.outgoing_ports.iter() {
            let target = target.upgrade().unwrap();
            let target = target.read().await;
            let target = target.as_any().downcast_ref::<HydroflowCrate>().unwrap();
            all_ports.insert(
                port_name.clone(),
                target.allocated_incoming.get(target_port).unwrap().clone(),
            );
        }

        let formatted_ports = all_ports
            .iter()
            .map(|(port_name, pipe)| format!("{}={:?}", port_name, pipe))
            .collect::<Vec<_>>()
            .join(";");

        self.launched_binary
            .as_mut()
            .unwrap()
            .write()
            .await
            .stdin()
            .send(format!("{formatted_ports}\n"))
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
