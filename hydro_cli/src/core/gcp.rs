use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use async_channel::{Receiver, Sender};

use async_ssh2_lite::{AsyncChannel, AsyncSession, SessionConfiguration};
use async_trait::async_trait;
use futures::{AsyncWriteExt, Future, StreamExt};
use hydroflow::util::connection::BindType;
use serde_json::json;
use tokio::{net::TcpStream, sync::RwLock};

use super::{
    localhost::create_broadcast,
    terraform::{TerraformOutput, TerraformProvider},
    ConnectionType, Host, LaunchedBinary, LaunchedHost, ResourceBatch, ResourceResult,
};

struct LaunchedComputeEngineBinary {
    _resource_result: Arc<ResourceResult>,
    channel: AsyncChannel<TcpStream>,
    stdin_channel: Sender<String>,
    stdout_receivers: Arc<RwLock<Vec<Sender<String>>>>,
    stderr_receivers: Arc<RwLock<Vec<Sender<String>>>>,
}

#[async_trait]
impl LaunchedBinary for LaunchedComputeEngineBinary {
    async fn stdin(&self) -> Sender<String> {
        self.stdin_channel.clone()
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
        if self.channel.eof() {
            self.channel.exit_status().ok()
        } else {
            None
        }
    }
}

struct LaunchedComputeEngine {
    resource_result: Arc<ResourceResult>,
    ip: String,
    binary_counter: RwLock<usize>,
}

async fn async_retry<T, F: Future<Output = Result<T>>>(
    thunk: impl Fn() -> F,
    count: usize,
) -> Result<T> {
    for _ in 1..count {
        let result = thunk().await;
        if result.is_ok() {
            return result;
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            println!("retrying...")
        }
    }

    thunk().await
}

#[async_trait]
impl LaunchedHost for LaunchedComputeEngine {
    async fn launch_binary(&self, binary: &Path) -> Result<Arc<RwLock<dyn LaunchedBinary>>> {
        let mut session = async_retry(
            || async {
                let mut config = SessionConfiguration::new();
                config.set_timeout(5000);
                AsyncSession::<TcpStream>::connect(
                    SocketAddr::new(self.ip.parse().unwrap(), 22),
                    Some(config),
                )
                .await
                .map_err(|e| anyhow!("failed to connect to server: {}", e))
            },
            10,
        )
        .await?;

        session.handshake().await.context("failed to handshake")?;

        session
            .userauth_pubkey_file(
                "hydro",
                None,
                self.resource_result
                    .terraform
                    .deployment_folder
                    .path()
                    .join(".ssh")
                    .join("vm_instance_ssh_key_pem")
                    .as_path(),
                None,
            )
            .await
            .map_err(|e| anyhow!("failed to authenticate with public key: {}", e))?;

        let sftp = session
            .sftp()
            .await
            .context("could not open sftp channel")?;

        let mut binary_counter_write = self.binary_counter.write().await;
        let my_binary_counter = *binary_counter_write;
        *binary_counter_write += 1;
        drop(binary_counter_write);

        let binary_path = PathBuf::from(format!("/home/hydro/hydro-{my_binary_counter}"));

        let mut created_file = sftp.create(&binary_path).await?;
        created_file
            .write_all(std::fs::read(binary).unwrap().as_slice())
            .await
            .context("failed to copy binary")?;

        let mut orig_file_stat = sftp.stat(&binary_path).await?;
        orig_file_stat.perm = Some(0o755);
        created_file.setstat(orig_file_stat).await?;
        created_file.close().await?;
        drop(created_file);

        let mut channel = session
            .channel_session()
            .await
            .context("could not open stdout channel")?;
        channel
            .exec(binary_path.to_str().unwrap())
            .await
            .context("could not launch binary")?;

        let (stdin_sender, mut stdin_receiver) = async_channel::unbounded::<String>();
        let mut stdin = channel.stream(0);
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
            stdin_channel: stdin_sender,
            stdout_receivers,
            stderr_receivers,
        })))
    }
}

pub struct GCPComputeEngineHost {
    pub id: usize,
    pub project: String,
    pub ip: Option<String>,
    pub launched: Option<Arc<dyn LaunchedHost>>,
}

#[async_trait]
impl Host for GCPComputeEngineHost {
    async fn collect_resources(&mut self, resource_batch: &mut ResourceBatch) {
        let project = self.project.as_str();
        let id = self.id;

        let _ = resource_batch
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

        let _ = resource_batch
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

        let _ = resource_batch
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

        let _ = resource_batch
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

        let _ = resource_batch
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

        let vpc_network = format!("vpc-network-{project}");
        let _ = resource_batch
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

        let allow_ssh_rule = format!("allow-ssh-{project}");
        let _ = resource_batch
            .terraform
            .resource
            .entry("google_compute_firewall".to_string())
            .or_default()
            .insert(
                allow_ssh_rule.clone(),
                json!({
                    "name": allow_ssh_rule,
                    "project": project,
                    "network": format!("${{google_compute_network.{vpc_network}.name}}"),
                    "target_tags": [allow_ssh_rule],
                    "source_ranges": ["0.0.0.0/0"],
                    "allow": [
                        {
                            "protocol": "tcp",
                            "ports": ["22"]
                        }
                    ]
                }),
            );

        let vm_instance = format!("vm-instance-{project}-{id}");
        let _ = resource_batch.terraform.resource.entry("google_compute_instance".to_string())
            .or_default()
            .insert(vm_instance.clone(), json!({
                "name": vm_instance,
                "project": project,
                "machine_type": "e2-micro",
                "zone": "us-west1-a",
                "tags": [ allow_ssh_rule ],
                "metadata": {
                "ssh-keys": "hydro:${tls_private_key.vm_instance_ssh_key.public_key_openssh}"
                },
                "boot_disk": [
                    {
                        "initialize_params": [
                            {
                                "image": "debian-cloud/debian-11"
                            }
                        ]
                    }
                ],
                "network_interface": [
                    {
                        "network": format!("${{google_compute_network.{vpc_network}.self_link}}"),
                        "access_config": [
                            {
                                "network_tier": "STANDARD"
                            }
                        ]
                    }
                ]
            }));

        let _ = resource_batch.terraform.output.insert(
            format!("{vm_instance}-public-ip"),
            TerraformOutput {
                value: format!("${{google_compute_instance.{vm_instance}.network_interface[0].access_config[0].nat_ip}}")
            }
        );
    }

    async fn provision(&mut self, resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        if self.launched.is_none() {
            let project = self.project.as_str();
            let id = self.id;

            let ip = &resource_result
                .terraform
                .outputs
                .get(&format!("vm-instance-{project}-{id}-public-ip"))
                .unwrap()
                .value;
            self.ip = Some(ip.clone());

            self.launched = Some(Arc::new(LaunchedComputeEngine {
                resource_result: resource_result.clone(),
                ip: ip.clone(),
                binary_counter: RwLock::new(0),
            }))
        }

        self.launched.as_ref().unwrap().clone()
    }

    fn find_bind_type(&self, connection_from: &dyn Host) -> BindType {
        if connection_from.can_connect_to(ConnectionType::UnixSocket(self.id)) {
            BindType::UnixSocket
        } else if connection_from.can_connect_to(ConnectionType::InternalTcpPort(self.id)) {
            BindType::TcpPort(self.ip.as_ref().unwrap().clone())
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
                    false
                }
            }
            ConnectionType::InternalTcpPort(id) => self.id == id,
        }
    }
}
