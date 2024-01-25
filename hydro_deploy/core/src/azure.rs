use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_ssh2_lite::{AsyncSession, SessionConfiguration};
use async_trait::async_trait;
use hydroflow_cli_integration::ServerBindConfig;
use nanoid::nanoid;
use serde_json::json;
use tokio::net::TcpStream;

use super::progress::ProgressTracker;
use super::ssh::LaunchedSSHHost;
use super::terraform::{TerraformOutput, TerraformProvider, TERRAFORM_ALPHABET};
use super::util::async_retry;
use super::{
    ClientStrategy, Host, HostTargetType, LaunchedHost, ResourceBatch, ResourceResult,
    ServerStrategy,
};

// #[derive(Debug)]
pub struct LaunchedComputeEngine {
    resource_result: Arc<ResourceResult>,
    user: String,
    pub internal_ip: String,
    pub external_ip: Option<String>,
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
}

#[async_trait]
impl LaunchedSSHHost for LaunchedComputeEngine {
    fn server_config(&self, bind_type: &ServerStrategy) -> ServerBindConfig {
        match bind_type {
            ServerStrategy::UnixSocket => ServerBindConfig::UnixSocket,
            ServerStrategy::InternalTcpPort => ServerBindConfig::TcpPort(self.internal_ip.clone()),
            ServerStrategy::ExternalTcpPort(_) => todo!(),
            ServerStrategy::Demux(demux) => {
                let mut config_map = HashMap::new();
                for (key, underlying) in demux {
                    config_map.insert(*key, LaunchedSSHHost::server_config(self, underlying));
                }

                ServerBindConfig::Demux(config_map)
            }
            ServerStrategy::Merge(merge) => {
                let mut configs = vec![];
                for underlying in merge {
                    configs.push(LaunchedSSHHost::server_config(self, underlying));
                }

                ServerBindConfig::Merge(configs)
            }
            ServerStrategy::Tagged(underlying, id) => ServerBindConfig::Tagged(
                Box::new(LaunchedSSHHost::server_config(self, underlying)),
                *id,
            ),
            ServerStrategy::Null => ServerBindConfig::Null,
        }
    }

    fn resource_result(&self) -> &Arc<ResourceResult> {
        &self.resource_result
    }

    fn ssh_user(&self) -> &str {
        self.user.as_str()
    }

    async fn open_ssh_session(&self) -> Result<AsyncSession<TcpStream>> {
        let target_addr = SocketAddr::new(
            self.external_ip
                .as_ref()
                .context("Azure host must be configured with an external IP to launch binaries")?
                .parse()
                .unwrap(),
            22,
        );

        let res = ProgressTracker::leaf(
            format!(
                "connecting to host @ {}",
                self.external_ip.as_ref().unwrap()
            ),
            async_retry(
                &|| async {
                    let mut config = SessionConfiguration::new();
                    config.set_compress(true);

                    let mut session =
                        AsyncSession::<TcpStream>::connect(target_addr, Some(config)).await?;

                    session.handshake().await?;

                    session
                        .userauth_pubkey_file(
                            self.user.as_str(),
                            None,
                            self.ssh_key_path().as_path(),
                            None,
                        )
                        .await?;

                    Ok(session)
                },
                10,
                Duration::from_secs(1),
            ),
        )
        .await?;

        Ok(res)
    }
}

pub struct AzureHost {
    pub id: usize,
    pub project: String,
    pub os_type: String, // linux or windows
    pub machine_size: String,
    pub image: String,
    pub region: String,
    pub user: Option<String>,
    pub launched: Option<Arc<LaunchedComputeEngine>>,
    external_ports: Vec<u16>,
}

impl AzureHost {
    pub fn new(
        id: usize,
        project: String,
        os_type: String, // linux or windows
        machine_size: String,
        image: String,
        region: String,
        user: Option<String>,
    ) -> Self {
        Self {
            id,
            project,
            os_type,
            machine_size,
            image,
            region,
            user,
            launched: None,
            external_ports: vec![],
        }
    }
}

#[async_trait]
impl Host for AzureHost {
    fn target_type(&self) -> HostTargetType {
        HostTargetType::Linux
    }

    fn request_port(&mut self, bind_type: &ServerStrategy) {
        match bind_type {
            ServerStrategy::UnixSocket => {}
            ServerStrategy::InternalTcpPort => {}
            ServerStrategy::ExternalTcpPort(port) => {
                if !self.external_ports.contains(port) {
                    if self.launched.is_some() {
                        todo!("Cannot adjust firewall after host has been launched");
                    }

                    self.external_ports.push(*port);
                }
            }
            ServerStrategy::Demux(demux) => {
                for bind_type in demux.values() {
                    self.request_port(bind_type);
                }
            }
            ServerStrategy::Merge(merge) => {
                for bind_type in merge {
                    self.request_port(bind_type);
                }
            }
            ServerStrategy::Tagged(underlying, _) => {
                self.request_port(underlying);
            }
            ServerStrategy::Null => {}
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn collect_resources(&self, resource_batch: &mut ResourceBatch) {
        if self.launched.is_some() {
            return;
        }

        let project = self.project.as_str();

        // first, we import the providers we need
        resource_batch
            .terraform
            .terraform
            .required_providers
            .insert(
                "azurerm".to_string(),
                TerraformProvider {
                    source: "hashicorp/azurerm".to_string(),
                    version: "3.67.0".to_string(),
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

        let vm_key = format!("vm-instance-{}", self.id);
        let vm_name = format!("hydro-vm-instance-{}", nanoid!(8, &TERRAFORM_ALPHABET));

        // Handle provider configuration
        resource_batch.terraform.provider.insert(
            "azurerm".to_string(),
            json!({
                "skip_provider_registration": "true",
                "features": {},
            }),
        );

        // Handle resources
        resource_batch
            .terraform
            .resource
            .entry("azurerm_resource_group".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": project,
                    "location": self.region.clone(),
                }),
            );

        resource_batch
            .terraform
            .resource
            .entry("azurerm_virtual_network".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": format!("{vm_key}-network"),
                    "address_space": ["10.0.0.0/16"],
                    "location": self.region.clone(),
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}")
                }),
            );

        resource_batch
            .terraform
            .resource
            .entry("azurerm_subnet".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": "internal",
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}"),
                    "virtual_network_name": format!("${{azurerm_virtual_network.{vm_key}.name}}"),
                    "address_prefixes": ["10.0.2.0/24"]
                }),
            );

        resource_batch
            .terraform
            .resource
            .entry("azurerm_public_ip".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": "hydropubip",
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}"),
                    "location": format!("${{azurerm_resource_group.{vm_key}.location}}"),
                    "allocation_method": "Static",
                }),
            );

        resource_batch
            .terraform
            .resource
            .entry("azurerm_network_interface".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": format!("{vm_key}-nic"),
                    "location": format!("${{azurerm_resource_group.{vm_key}.location}}"),
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}"),
                    "ip_configuration": {
                        "name": "internal",
                        "subnet_id": format!("${{azurerm_subnet.{vm_key}.id}}"),
                        "private_ip_address_allocation": "Dynamic",
                        "public_ip_address_id": format!("${{azurerm_public_ip.{vm_key}.id}}"),
                    }
                }),
            );

        // Define network security rules - for now, accept all connections
        resource_batch
            .terraform
            .resource
            .entry("azurerm_network_security_group".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": "primary_security_group",
                    "location": format!("${{azurerm_resource_group.{vm_key}.location}}"),
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}"),
                }),
            );

        resource_batch
            .terraform
            .resource
            .entry("azurerm_network_security_rule".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "name": "allowall",
                    "priority": 100,
                    "direction": "Inbound",
                    "access": "Allow",
                    "protocol": "Tcp",
                    "source_port_range": "*",
                    "destination_port_range": "*",
                    "source_address_prefix": "*",
                    "destination_address_prefix": "*",
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}"),
                    "network_security_group_name": format!("${{azurerm_network_security_group.{vm_key}.name}}"),
                })
            );

        resource_batch
            .terraform
            .resource
            .entry("azurerm_subnet_network_security_group_association".to_string())
            .or_default()
            .insert(
                vm_key.to_string(),
                json!({
                    "subnet_id": format!("${{azurerm_subnet.{vm_key}.id}}"),
                    "network_security_group_id": format!("${{azurerm_network_security_group.{vm_key}.id}}"),
                })
            );

        let user = self.user.as_ref().cloned().unwrap_or("hydro".to_string());
        let os_type = format!("azurerm_{}_virtual_machine", self.os_type.clone());
        resource_batch
            .terraform
            .resource
            .entry(os_type.clone())
            .or_default()
            .insert(
                vm_key.clone(),
                json!({
                    "name": vm_name,
                    "resource_group_name": format!("${{azurerm_resource_group.{vm_key}.name}}"),
                    "location": format!("${{azurerm_resource_group.{vm_key}.location}}"),
                    "size": self.machine_size.clone(),
                    "network_interface_ids": [format!("${{azurerm_network_interface.{vm_key}.id}}")],
                    "admin_ssh_key": {
                        "username": user,
                        "public_key": "${tls_private_key.vm_instance_ssh_key.public_key_openssh}",
                    },
                    "admin_username": user,
                    "os_disk": {
                        "caching": "ReadWrite",
                        "storage_account_type": "Standard_LRS",
                    },
                    "source_image_reference": {
                        "publisher": "Canonical",
                        "offer": "0001-com-ubuntu-server-jammy",
                        "sku": "22_04-lts",
                        "version": "latest",
                    }
                }),
            );

        resource_batch.terraform.output.insert(
            format!("{vm_key}-public-ip"),
            TerraformOutput {
                value: format!("${{azurerm_public_ip.{vm_key}.ip_address}}"),
            },
        );

        resource_batch.terraform.output.insert(
            format!("{vm_key}-internal-ip"),
            TerraformOutput {
                value: format!("${{azurerm_network_interface.{vm_key}.private_ip_address}}"),
            },
        );
    }

    fn launched(&self) -> Option<Arc<dyn LaunchedHost>> {
        self.launched
            .as_ref()
            .map(|a| a.clone() as Arc<dyn LaunchedHost>)
    }

    async fn provision(&mut self, resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        if self.launched.is_none() {
            let id = self.id;

            let internal_ip = resource_result
                .terraform
                .outputs
                .get(&format!("vm-instance-{id}-internal-ip"))
                .unwrap()
                .value
                .clone();

            let external_ip = resource_result
                .terraform
                .outputs
                .get(&format!("vm-instance-{id}-public-ip"))
                .map(|v| v.value.clone());

            self.launched = Some(Arc::new(LaunchedComputeEngine {
                resource_result: resource_result.clone(),
                user: self.user.as_ref().cloned().unwrap_or("hydro".to_string()),
                internal_ip,
                external_ip,
            }))
        }

        self.launched.as_ref().unwrap().clone()
    }

    fn strategy_as_server<'a>(
        &'a self,
        client_host: &dyn Host,
    ) -> Result<(
        ClientStrategy<'a>,
        Box<dyn FnOnce(&mut dyn std::any::Any) -> ServerStrategy>,
    )> {
        if client_host.can_connect_to(ClientStrategy::UnixSocket(self.id)) {
            Ok((
                ClientStrategy::UnixSocket(self.id),
                Box::new(|_| ServerStrategy::UnixSocket),
            ))
        } else if client_host.can_connect_to(ClientStrategy::InternalTcpPort(self)) {
            Ok((
                ClientStrategy::InternalTcpPort(self),
                Box::new(|_| ServerStrategy::InternalTcpPort),
            ))
        } else if client_host.can_connect_to(ClientStrategy::ForwardedTcpPort(self)) {
            Ok((
                ClientStrategy::ForwardedTcpPort(self),
                Box::new(|me| {
                    me.downcast_mut::<AzureHost>()
                        .unwrap()
                        .request_port(&ServerStrategy::ExternalTcpPort(22)); // needed to forward
                    ServerStrategy::InternalTcpPort
                }),
            ))
        } else {
            anyhow::bail!("Could not find a strategy to connect to Azure instance")
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
                if let Some(provider_target) = target_host.as_any().downcast_ref::<AzureHost>() {
                    self.project == provider_target.project
                } else {
                    false
                }
            }
            ClientStrategy::ForwardedTcpPort(_) => false,
        }
    }
}
