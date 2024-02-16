// use std::collections::HashMap;
// use std::net::SocketAddr;
// use std::path::PathBuf;
// use std::sync::Arc;
// use std::time::Duration;

// use anyhow::{Context, Result};
// use async_ssh2_lite::{AsyncSession, SessionConfiguration};
// use async_trait::async_trait;
// use hydroflow_cli_integration::ServerBindConfig;
// use tokio::net::TcpStream;

// use super::progress::ProgressTracker;
// use super::ssh::LaunchedSSHHost;
// use super::util::async_retry;
// use super::{
//     ResourceResult,
//     ServerStrategy,
// };

// pub struct LaunchedVirtualMachine {
//   pub cloud_provider: String, // gcp or azure
//   pub resource_result: Arc<ResourceResult>,
//   pub user: String,
//   pub internal_ip: String,
//   pub external_ip: Option<String>,
// }

// impl LaunchedVirtualMachine {
//   pub fn ssh_key_path(&self) -> PathBuf {
//       self.resource_result
//           .terraform
//           .deployment_folder
//           .as_ref()
//           .unwrap()
//           .path()
//           .join(".ssh")
//           .join("vm_instance_ssh_key_pem")
//   }
// }

// #[async_trait]
// impl LaunchedSSHHost for LaunchedVirtualMachine {
//   fn server_config(&self, bind_type: &ServerStrategy) -> ServerBindConfig {
//       match bind_type {
//           ServerStrategy::UnixSocket => ServerBindConfig::UnixSocket,
//           ServerStrategy::InternalTcpPort => ServerBindConfig::TcpPort(self.internal_ip.clone()),
//           ServerStrategy::ExternalTcpPort(_) => todo!(),
//           ServerStrategy::Demux(demux) => {
//               let mut config_map = HashMap::new();
//               for (key, underlying) in demux {
//                   config_map.insert(*key, LaunchedSSHHost::server_config(self, underlying));
//               }

//               ServerBindConfig::Demux(config_map)
//           }
//           ServerStrategy::Merge(merge) => {
//               let mut configs = vec![];
//               for underlying in merge {
//                   configs.push(LaunchedSSHHost::server_config(self, underlying));
//               }

//               ServerBindConfig::Merge(configs)
//           }
//           ServerStrategy::Tagged(underlying, id) => ServerBindConfig::Tagged(
//               Box::new(LaunchedSSHHost::server_config(self, underlying)),
//               *id,
//           ),
//           ServerStrategy::Null => ServerBindConfig::Null,
//       }
//   }

//   fn resource_result(&self) -> &Arc<ResourceResult> {
//       &self.resource_result
//   }

//   fn ssh_user(&self) -> &str {
//       self.user.as_str()
//   }

//   async fn open_ssh_session(&self) -> Result<AsyncSession<TcpStream>> {
//       let target_addr = SocketAddr::new(
//           self.external_ip
//               .as_ref()
//               .context(self.cloud_provider.clone() + " Host must be configured with an external IP to launch binaries")?
//               .parse()
//               .unwrap(),
//           22,
//       );

//       let res = ProgressTracker::leaf(
//           format!(
//               "connecting to host @ {}",
//               self.external_ip.as_ref().unwrap()
//           ),
//           async_retry(
//               &|| async {
//                   let mut config = SessionConfiguration::new();
//                   config.set_compress(true);

//                   let mut session =
//                       AsyncSession::<TcpStream>::connect(target_addr, Some(config)).await?;

//                   session.handshake().await?;

//                   session
//                       .userauth_pubkey_file(
//                           self.user.as_str(),
//                           None,
//                           self.ssh_key_path().as_path(),
//                           None,
//                       )
//                       .await?;

//                   Ok(session)
//               },
//               10,
//               Duration::from_secs(1),
//           ),
//       )
//       .await?;

//       Ok(res)
//   }
// }