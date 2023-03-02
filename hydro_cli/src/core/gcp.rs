use std::{
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
use hydroflow::util::connection::BindConfig;
use serde_json::json;
use tokio::{net::TcpStream, sync::RwLock};

use super::{
    localhost::create_broadcast,
    terraform::{TerraformOutput, TerraformProvider},
    util::async_retry,
    BindType, ConnectionType, Host, LaunchedBinary, LaunchedHost, ResourceBatch, ResourceResult,
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
            .path()
            .join(".ssh")
            .join("vm_instance_ssh_key_pem")
    }
}

#[async_trait]
impl LaunchedHost for LaunchedComputeEngine {
    fn get_bind_config(&self, bind_type: &BindType) -> BindConfig {
        match bind_type {
            BindType::UnixSocket => BindConfig::UnixSocket,
            BindType::InternalTcpPort => BindConfig::TcpPort(self.internal_ip.clone()),
            BindType::ExternalTcpPort(_) => todo!(),
        }
    }

    async fn launch_binary(&self, binary: &Path) -> Result<Arc<RwLock<dyn LaunchedBinary>>> {
        let target_addr = SocketAddr::new(
            self.external_ip
                .as_ref()
                .context("GCP host must be configured with an external IP to launch binaries")?
                .parse()
                .unwrap(),
            22,
        );
        let session = async_retry(
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
        .await?;

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
        channel.exec(binary_path.to_str().unwrap()).await?;

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
    fn request_port(&mut self, bind_type: &BindType) {
        match bind_type {
            BindType::UnixSocket => {}
            BindType::InternalTcpPort => {}
            BindType::ExternalTcpPort(port) => {
                self.external_ports.push(*port);
            }
        }
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

    fn get_bind_type(&self, connection_from: &dyn Host) -> BindType {
        if connection_from.can_connect_to(ConnectionType::UnixSocket(self.id)) {
            BindType::UnixSocket
        } else if connection_from
            .can_connect_to(ConnectionType::InternalTcpPort(self.project.clone()))
        {
            BindType::InternalTcpPort
        } else {
            todo!()
        }
    }

    fn can_connect_to(&self, typ: ConnectionType) -> bool {
        match typ {
            ConnectionType::UnixSocket(id) => {
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
            ConnectionType::InternalTcpPort(id) => self.project == id,
        }
    }
}
