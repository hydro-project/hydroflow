use std::{
    borrow::Cow,
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use anyhow::{Context, Result};
use async_channel::{Receiver, Sender};

use async_ssh2_lite::{AsyncChannel, AsyncSession, SessionConfiguration};
use async_trait::async_trait;
use futures::{AsyncWriteExt, StreamExt};
use hydroflow_cli_integration::ServerConfig;
use serde_json::json;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use super::{
    localhost::create_broadcast,
    terraform::{TerraformOutput, TerraformProvider},
    util::async_retry,
    ClientStrategy, Host, HostTargetType, LaunchedBinary, LaunchedHost, ResourceBatch,
    ResourceResult, ServerStrategy,
};

struct LaunchedComputeEngineBinary {
    _resource_result: Arc<ResourceResult>,
    channel: AsyncChannel<TcpStream>,
    stdin_sender: Sender<String>,
    stdout_receivers: Arc<RwLock<Vec<Sender<String>>>>,
    stderr_receivers: Arc<RwLock<Vec<Sender<String>>>>,
}

#[async_trait]
impl LaunchedBinary for LaunchedComputeEngineBinary {
    async fn stdin(&self) -> Sender<String> {
        self.stdin_sender.clone()
    }

    async fn stdout(&self) -> Receiver<String> {
        let mut receivers = self.stdout_receivers.write().await;
        let (sender, receiver) = async_channel::unbounded::<String>();
        receivers.push(sender);
        receiver
    }

    async fn stderr(&self) -> Receiver<String> {
        let mut receivers = self.stderr_receivers.write().await;
        let (sender, receiver) = async_channel::unbounded::<String>();
        receivers.push(sender);
        receiver
    }

    async fn exit_code(&self) -> Option<i32> {
        // until the program exits, the exit status is meaningless
        if self.channel.eof() {
            self.channel.exit_status().ok()
        } else {
            None
        }
    }
}

pub struct LaunchedComputeEngine {
    resource_result: Arc<ResourceResult>,
    pub internal_ip: String,
    pub external_ip: Option<String>,
    binary_counter: AtomicUsize,
}

impl LaunchedComputeEngine {
    pub fn ssh_key_path(&self) -> PathBuf {
        self.resource_result
            .terraform
            .deployment_folder
            .as_ref()
            .unwrap()
            .path()
            .join(".ssh")
            .join("vm_instance_ssh_key_pem")
    }

    async fn open_ssh_session(&self) -> Result<AsyncSession<TcpStream>> {
        let target_addr = SocketAddr::new(
            self.external_ip
                .as_ref()
                .context("GCP host must be configured with an external IP to launch binaries")?
                .parse()
                .unwrap(),
            22,
        );

        async_retry(
            || async {
                let mut config = SessionConfiguration::new();
                config.set_timeout(5000);

                let mut session =
                    AsyncSession::<TcpStream>::connect(target_addr, Some(config)).await?;

                session.handshake().await?;

                session
                    .userauth_pubkey_file("hydro", None, self.ssh_key_path().as_path(), None)
                    .await?;

                Ok(session)
            },
            10,
            Duration::from_secs(1),
        )
        .await
    }
}

#[async_trait]
impl LaunchedHost for LaunchedComputeEngine {
    fn server_config(&self, bind_type: &ServerStrategy) -> ServerConfig {
        match bind_type {
            ServerStrategy::UnixSocket => ServerConfig::UnixSocket,
            ServerStrategy::InternalTcpPort => ServerConfig::TcpPort(self.internal_ip.clone()),
            ServerStrategy::ExternalTcpPort(_) => todo!(),
            ServerStrategy::Demux(demux) => {
                let mut config_map = HashMap::new();
                for (key, bind_type) in demux {
                    config_map.insert(*key, self.server_config(bind_type));
                }

                ServerConfig::Demux(config_map)
            }
        }
    }

    async fn launch_binary(
        &self,
        binary: &Path,
        args: &[String],
    ) -> Result<Arc<RwLock<dyn LaunchedBinary>>> {
        let session = self.open_ssh_session().await?;

        let sftp = session.sftp().await?;

        // we may be deploying multiple binaries, so give each a unique name
        let my_binary_counter = self
            .binary_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let binary_path = PathBuf::from(format!("/home/hydro/hydro-{my_binary_counter}"));

        let mut created_file = sftp.create(&binary_path).await?;
        created_file
            .write_all(std::fs::read(binary).unwrap().as_slice())
            .await?;

        let mut orig_file_stat = sftp.stat(&binary_path).await?;
        orig_file_stat.perm = Some(0o755); // allow the copied binary to be executed by anyone
        created_file.setstat(orig_file_stat).await?;
        created_file.close().await?;
        drop(created_file);

        let mut channel = session.channel_session().await?;
        let binary_path_string = binary_path.to_str().unwrap();
        let args_string = args
            .iter()
            .map(|s| shell_escape::unix::escape(Cow::from(s)))
            .fold("".to_string(), |acc, v| format!("{acc} {v}"));
        channel
            .exec(&format!("{binary_path_string}{args_string}"))
            .await?;

        let (stdin_sender, mut stdin_receiver) = async_channel::unbounded::<String>();
        let mut stdin = channel.stream(0); // stream 0 is stdout/stdin, we use it for stdin
        tokio::spawn(async move {
            while let Some(line) = stdin_receiver.next().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let stdout_receivers = create_broadcast(channel.stream(0), |s| println!("{s}"));
        let stderr_receivers = create_broadcast(channel.stderr(), |s| eprintln!("{s}"));

        Ok(Arc::new(RwLock::new(LaunchedComputeEngineBinary {
            _resource_result: self.resource_result.clone(),
            channel,
            stdin_sender,
            stdout_receivers,
            stderr_receivers,
        })))
    }

    async fn forward_port(&self, port: u16) -> Result<SocketAddr> {
        let session = self.open_ssh_session().await?;

        let local_port = TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = local_port.local_addr()?;

        let internal_ip = self.internal_ip.clone();

        tokio::spawn(async move {
            while let Ok((mut local_stream, _)) = local_port.accept().await {
                let mut channel = session
                    .channel_direct_tcpip(&internal_ip, port, None)
                    .await
                    .unwrap();
                let _ = tokio::io::copy_bidirectional(&mut local_stream, &mut channel).await;
            }
        });

        Ok(local_addr)
    }
}

pub struct GCPComputeEngineHost {
    pub id: usize,
    pub project: String,
    pub machine_type: String,
    pub image: String,
    pub region: String,
    pub launched: Option<Arc<LaunchedComputeEngine>>,
    external_ports: Vec<u16>,
}

impl GCPComputeEngineHost {
    pub fn new(
        id: usize,
        project: String,
        machine_type: String,
        image: String,
        region: String,
    ) -> Self {
        Self {
            id,
            project,
            machine_type,
            image,
            region,
            launched: None,
            external_ports: vec![],
        }
    }
}

#[async_trait]
impl Host for GCPComputeEngineHost {
    fn target_type(&self) -> HostTargetType {
        HostTargetType::Linux
    }

    fn request_port(&mut self, bind_type: &ServerStrategy) {
        match bind_type {
            ServerStrategy::UnixSocket => {}
            ServerStrategy::InternalTcpPort => {}
            ServerStrategy::ExternalTcpPort(port) => {
                self.external_ports.push(*port);
            }
            ServerStrategy::Demux(demux) => {
                for bind_type in demux.values() {
                    self.request_port(bind_type);
                }
            }
        }
    }

    fn request_custom_binary(&mut self) {
        self.request_port(&ServerStrategy::ExternalTcpPort(22));
    }

    fn id(&self) -> usize {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn collect_resources(&self, resource_batch: &mut ResourceBatch) {
        let project = self.project.as_str();
        let id = self.id;

        // first, we import the providers we need
        resource_batch
            .terraform
            .terraform
            .required_providers
            .insert(
                "google".to_string(),
                TerraformProvider {
                    source: "hashicorp/google".to_string(),
                    version: "4.53.1".to_string(),
                },
            );

        resource_batch
            .terraform
            .terraform
            .required_providers
            .insert(
                "local".to_string(),
                TerraformProvider {
                    source: "hashicorp/local".to_string(),
                    version: "2.3.0".to_string(),
                },
            );

        resource_batch
            .terraform
            .terraform
            .required_providers
            .insert(
                "tls".to_string(),
                TerraformProvider {
                    source: "hashicorp/tls".to_string(),
                    version: "4.0.4".to_string(),
                },
            );

        // we use a single SSH key for all VMs
        resource_batch
            .terraform
            .resource
            .entry("tls_private_key".to_string())
            .or_default()
            .insert(
                "vm_instance_ssh_key".to_string(),
                json!({
                    "algorithm": "RSA",
                    "rsa_bits": 4096
                }),
            );

        resource_batch
            .terraform
            .resource
            .entry("local_file".to_string())
            .or_default()
            .insert(
                "vm_instance_ssh_key_pem".to_string(),
                json!({
                    "content": "${tls_private_key.vm_instance_ssh_key.private_key_pem}",
                    "filename": ".ssh/vm_instance_ssh_key_pem",
                    "file_permission": "0600"
                }),
            );

        // we use a single VPC for all VMs
        let vpc_network = format!("vpc-network-{project}");
        resource_batch
            .terraform
            .resource
            .entry("google_compute_network".to_string())
            .or_default()
            .insert(
                vpc_network.clone(),
                json!({
                    "name": vpc_network,
                    "project": project,
                    "auto_create_subnetworks": true
                }),
            );

        let firewall_entries = resource_batch
            .terraform
            .resource
            .entry("google_compute_firewall".to_string())
            .or_default();

        // allow all VMs to communicate with each other over internal IPs
        firewall_entries.insert(
            format!("{vpc_network}-default-allow-internal"),
            json!({
                "name": format!("{vpc_network}-default-allow-internal"),
                "project": project,
                "network": format!("${{google_compute_network.{vpc_network}.name}}"),
                "source_ranges": ["10.128.0.0/9"],
                "allow": [
                    {
                        "protocol": "tcp",
                        "ports": ["0-65535"]
                    },
                    {
                        "protocol": "udp",
                        "ports": ["0-65535"]
                    },
                    {
                        "protocol": "icmp"
                    }
                ]
            }),
        );

        // allow external pings to all VMs
        firewall_entries.insert(
            format!("{vpc_network}-default-allow-ping"),
            json!({
                "name": format!("{vpc_network}-default-allow-ping"),
                "project": project,
                "network": format!("${{google_compute_network.{vpc_network}.name}}"),
                "source_ranges": ["0.0.0.0/0"],
                "allow": [
                    {
                        "protocol": "icmp"
                    }
                ]
            }),
        );

        let vm_instance = format!("vm-instance-{project}-{id}");

        let mut tags = vec![];
        let mut external_interfaces = vec![];

        if self.external_ports.is_empty() {
            external_interfaces.push(json!({
                "network": format!("${{google_compute_network.{vpc_network}.self_link}}")
            }));
        } else {
            external_interfaces.push(json!({
                "network": format!("${{google_compute_network.{vpc_network}.self_link}}"),
                "access_config": [
                    {
                        "network_tier": "STANDARD"
                    }
                ]
            }));

            // open the external ports that were requested
            let open_external_ports_rule = format!("{vm_instance}-open-external-ports");
            firewall_entries.insert(
                open_external_ports_rule.clone(),
                json!({
                    "name": open_external_ports_rule,
                    "project": project,
                    "network": format!("${{google_compute_network.{vpc_network}.name}}"),
                    "target_tags": [open_external_ports_rule],
                    "source_ranges": ["0.0.0.0/0"],
                    "allow": [
                        {
                            "protocol": "tcp",
                            "ports": self.external_ports.iter().map(|p| p.to_string()).collect::<Vec<String>>()
                        }
                    ]
                }),
            );

            tags.push(open_external_ports_rule);

            resource_batch.terraform.output.insert(
                format!("{vm_instance}-public-ip"),
                TerraformOutput {
                    value: format!("${{google_compute_instance.{vm_instance}.network_interface[0].access_config[0].nat_ip}}")
                }
            );
        }

        resource_batch
            .terraform
            .resource
            .entry("google_compute_instance".to_string())
            .or_default()
            .insert(
                vm_instance.clone(),
                json!({
                    "name": vm_instance,
                    "project": project,
                    "machine_type": self.machine_type,
                    "zone": self.region,
                    "tags": tags,
                    "metadata": {
                    "ssh-keys": "hydro:${tls_private_key.vm_instance_ssh_key.public_key_openssh}"
                    },
                    "boot_disk": [
                        {
                            "initialize_params": [
                                {
                                    "image": self.image
                                }
                            ]
                        }
                    ],
                    "network_interface": external_interfaces
                }),
            );

        resource_batch.terraform.output.insert(
            format!("{vm_instance}-internal-ip"),
            TerraformOutput {
                value: format!(
                    "${{google_compute_instance.{vm_instance}.network_interface[0].network_ip}}"
                ),
            },
        );
    }

    async fn provision(&mut self, resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        if self.launched.is_none() {
            let project = self.project.as_str();
            let id = self.id;

            let internal_ip = resource_result
                .terraform
                .outputs
                .get(&format!("vm-instance-{project}-{id}-internal-ip"))
                .unwrap()
                .value
                .clone();

            let external_ip = resource_result
                .terraform
                .outputs
                .get(&format!("vm-instance-{project}-{id}-public-ip"))
                .map(|v| v.value.clone());

            self.launched = Some(Arc::new(LaunchedComputeEngine {
                resource_result: resource_result.clone(),
                internal_ip,
                external_ip,
                binary_counter: AtomicUsize::new(0),
            }))
        }

        self.launched.as_ref().unwrap().clone()
    }

    fn strategy_as_server(
        &mut self,
        client_host: Option<&dyn Host>,
    ) -> Result<(ClientStrategy, ServerStrategy)> {
        let client_host = client_host.unwrap_or(self);
        if client_host.can_connect_to(ClientStrategy::UnixSocket(self.id)) {
            Ok((
                ClientStrategy::UnixSocket(self.id),
                ServerStrategy::UnixSocket,
            ))
        } else if client_host.can_connect_to(ClientStrategy::InternalTcpPort(self)) {
            Ok((
                ClientStrategy::InternalTcpPort(self),
                ServerStrategy::InternalTcpPort,
            ))
        } else if client_host.can_connect_to(ClientStrategy::ForwardedTcpPort(self)) {
            self.request_port(&ServerStrategy::ExternalTcpPort(22)); // needed to forward
            Ok((
                ClientStrategy::ForwardedTcpPort(self),
                ServerStrategy::InternalTcpPort,
            ))
        } else {
            anyhow::bail!("Could not find a strategy to connect to GCP instance")
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
            ClientStrategy::InternalTcpPort(target_host) => {
                if let Some(gcp_target) =
                    target_host.as_any().downcast_ref::<GCPComputeEngineHost>()
                {
                    self.project == gcp_target.project
                } else {
                    false
                }
            }
            ClientStrategy::ForwardedTcpPort(_) => false,
        }
    }
}
