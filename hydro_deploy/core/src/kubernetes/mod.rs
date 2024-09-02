use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use hydroflow_cli_integration::ServerBindConfig;
use tokio::sync::RwLock;
use std::fs::Metadata;

use super::{
    ClientStrategy, Host, HostTargetType, LaunchedBinary, LaunchedHost, ResourceBatch,
    ResourceResult, ServerStrategy,
};

use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, AttachParams, ListParams, PostParams, ResourceExt, WatchEvent, WatchParams},
    Client,
};
use std::time::Duration;
use tokio::time::sleep;

use tokio::io::AsyncWriteExt;
use nanoid::nanoid;

use super::progress::ProgressTracker;

pub mod launched_binary;
pub use launched_binary::*;

#[derive(Debug)]
pub struct PodHost {
    pub id: usize,
    client_only: bool,
    launched: Option<Arc<LaunchedPod>>,
}

impl PodHost {
    pub fn new(id: usize) -> PodHost {
        PodHost {
            id,
            client_only: false,
            launched: None,
        }
    }

    pub fn client_only(&self) -> PodHost {
        PodHost {
            id: self.id,
            client_only: true,
            launched: None,
        }
    }
}

#[async_trait]
impl Host for PodHost {
    fn target_type(&self) -> HostTargetType {
        HostTargetType::Linux(crate::LinuxArchitecture::AARCH64)
    }

    fn request_port(&mut self, _bind_type: &ServerStrategy) {}
    fn collect_resources(&self, _resource_batch: &mut ResourceBatch) {}
    fn request_custom_binary(&mut self) {}

    fn id(&self) -> usize {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn launched(&self) -> Option<Arc<dyn LaunchedHost>> {
        self.launched
            .as_ref()
            .map(|a| a.clone() as Arc<dyn LaunchedHost>)
    }

    async fn provision(&mut self, _resource_result: &Arc<ResourceResult>) -> Arc<dyn LaunchedHost> {
        if self.launched.is_none() {
            let client = Client::try_default().await.unwrap();
            let alphabet = [
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p','q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0'
            ];
            let pod_id = nanoid!(10, &alphabet); // pod names can only contain alphanumeric characters
            let pod_name = format!("hydro-{}", pod_id);

            // Blank template for a new pod
            let p: Pod = serde_json::from_value(serde_json::json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "metadata": { "name": pod_name },
                "spec": {
                    "containers": [{
                        "name": pod_name,
                        "image": "ubuntu:20.04",
                        // Do nothing
                        "command": ["tail", "-f", "/dev/null"],
                    }],
                }
            })).unwrap();

            let pods: Api<Pod> = Api::default_namespaced(client);

            // Check if pod_name has already been created. If not, create it.
            let lp = ListParams::default().fields(&format!("metadata.name={}", pod_name)); // only want results for our pod
            let mut found_existing_pod = false;
            for p in pods.list(&lp).await.unwrap() {
                // ProgressTracker::println(format!("Found Pod: {}", p.name_any()).as_str());
                if p.name_any() == pod_name {
                    found_existing_pod = true;
                }
            }
            if !found_existing_pod {
                // ProgressTracker::println(format!("Creating new pod {:?}", pod_name).as_str());
                let res = pods.create(&PostParams::default(), &p).await;
                match res {
                    Err(e) => ProgressTracker::println(format!("{:?}", e).as_str()),
                    Ok(_) => (),
                }
            }

            // Wait until the pod is running, otherwise we get 500 error.
            let wp = WatchParams::default().fields(format!("metadata.name={}", pod_name).as_str()).timeout(10);
            let mut stream = pods.watch(&wp, "0").await.unwrap().boxed();
            while let Some(status) = stream.try_next().await.unwrap() {
                match status {
                    WatchEvent::Added(o) => {
                        ProgressTracker::println(&format!("Got {}", o.name_any()));
                    }
                    WatchEvent::Modified(o) => {
                        let s = o.status.as_ref().expect("status exists on pod");
                        if s.phase.clone().unwrap_or_default() == "Running" {
                            ProgressTracker::println(&format!("Ready to attach to {}", o.name_any()));
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let internal_ip = pods.get_status(pod_name.as_str()).await.unwrap().status.unwrap().pod_ip.unwrap();

            self.launched = Some(Arc::new(LaunchedPod {
                internal_ip,
                pod_name,
            }))

        }

        self.launched.as_ref().unwrap().clone()
    }

    fn strategy_as_server<'a>(
        &'a self,
        connection_from: &dyn Host,
    ) -> Result<(
        ClientStrategy<'a>,
        Box<dyn FnOnce(&mut dyn std::any::Any) -> ServerStrategy>,
    )> {
        if self.client_only {
            anyhow::bail!("Pod cannot be a server if it is client only")
        }

        if connection_from.can_connect_to(ClientStrategy::UnixSocket(self.id)) {
            Ok((
                ClientStrategy::UnixSocket(self.id),
                Box::new(|_| ServerStrategy::UnixSocket),
            ))
        } else if connection_from.can_connect_to(ClientStrategy::InternalTcpPort(self)) {
            Ok((
                ClientStrategy::InternalTcpPort(self),
                Box::new(|_| ServerStrategy::InternalTcpPort),
            ))
        } else {
            anyhow::bail!("Could not find a strategy to connect to Pod")
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
            // target_host.as_any().downcast_ref::<PodHost>()
            ClientStrategy::InternalTcpPort(_target_host) => true, // TODO: if I'm on the same cluster, can just return true first
            ClientStrategy::ForwardedTcpPort(_) => true,
        }
    }
}

#[derive(Debug)]
pub struct LaunchedPod {
    pub internal_ip: String, // internal IP address for inter-pod communication in a node
    pub pod_name: String,
}

#[async_trait]
impl LaunchedHost for LaunchedPod {
    fn server_config(&self, bind_type: &ServerStrategy) -> ServerBindConfig {
        match bind_type {
            ServerStrategy::UnixSocket => ServerBindConfig::UnixSocket,
            ServerStrategy::InternalTcpPort => ServerBindConfig::TcpPort(self.internal_ip.clone()), // TODO: change to pod's internal port
            ServerStrategy::ExternalTcpPort(_) => panic!("Cannot bind to external port"),
            ServerStrategy::Demux(demux) => {
                let mut config_map = HashMap::new();
                for (key, underlying) in demux {
                    config_map.insert(*key, self.server_config(underlying));
                }

                ServerBindConfig::Demux(config_map)
            }
            ServerStrategy::Merge(merge) => {
                let mut configs = vec![];
                for underlying in merge {
                    configs.push(self.server_config(underlying));
                }

                ServerBindConfig::Merge(configs)
            }
            ServerStrategy::Tagged(underlying, id) => {
                ServerBindConfig::Tagged(Box::new(self.server_config(underlying)), *id)
            }
            ServerStrategy::Null => ServerBindConfig::Null,
        }
    }

    async fn copy_binary(&self, binary: Arc<(String, Vec<u8>, PathBuf)>) -> Result<()> {
        // Create a new pod in the running kubernetes cluster (we assume the user already has one up)
        let client = Client::try_default().await?;
        let pods: Api<Pod> = Api::default_namespaced(client);

        let file_name = format!("hydro-{}-binary", binary.0);
        let metadata = std::fs::metadata("/Users/nickjiang/Nick/Hydro/hydroflow/hydro_deploy/core/src/kubernetes/hello_world_aarch64_musl")?;  // importantly, this file has executable permissions

        {
            let binary_data = binary.1.clone();
            let mut header = tar::Header::new_gnu();
            header.set_path(file_name).unwrap();
            header.set_size(binary_data.len() as u64);
            header.set_metadata(&metadata);
            header.set_cksum();

            let mut ar = tar::Builder::new(Vec::new());
            ar.append(&header, &mut binary_data.as_slice()).unwrap();
            let data = ar.into_inner().unwrap();

            let ap = AttachParams::default().stdin(true).stderr(false);
            let mut tar = pods
                .exec(self.pod_name.as_str(), vec!["tar", "xf", "-", "-C", "/"], &ap)
                .await?;
            let mut tar_stdin = tar.stdin().unwrap();
            tar_stdin.write_all(&data).await?;

            // Flush the stdin to finish sending the file through
            tar_stdin.flush().await?;

            tar.join().await?; // TODO: Do something with the result of this
        }

        Ok(())
    }

    async fn launch_binary(
        &self,
        id: String,
        binary: Arc<(String, Vec<u8>, PathBuf)>,
        args: &[String],
    ) -> Result<Arc<RwLock<dyn LaunchedBinary>>> {
        // ProgressTracker::println("Launching binary in Pod");

        let client = Client::try_default().await?;
        let pods: Api<Pod> = Api::default_namespaced(client);
        let pod_name = &self.pod_name;
        let file_name = format!("hydro-{}-binary", binary.0);

        // Construct arguments
        let mut args_list = vec![format!("./{file_name}")];
        args_list.extend(args.iter().cloned());

        // Execute binary inside the new pod
        let ap = AttachParams::default().stdin(true).stdout(true).stderr(true);
        let mut launch_binary = pods
            .exec(pod_name, args_list, &ap)
            .await?;

        Ok(Arc::new(RwLock::new(LaunchedPodBinary::new(
            &mut launch_binary, id,
        ))))
    }

    async fn forward_port(&self, addr: &SocketAddr) -> Result<SocketAddr> {
        Ok(*addr)
    }
}
