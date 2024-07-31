use std::collections::HashMap;
use std::sync::{Arc, Weak};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use tokio::sync::RwLock;

use super::gcp::GcpNetwork;
use super::{
    progress, CustomService, GcpComputeEngineHost, Host, LocalhostHost, ResourcePool,
    ResourceResult, Service,
};
use crate::{AzureHost, ServiceBuilder};

#[derive(Default)]
pub struct Deployment {
    pub hosts: Vec<Arc<dyn Host>>,
    pub services: Vec<Weak<RwLock<dyn Service>>>,
    pub resource_pool: ResourcePool,
    last_resource_result: Option<Arc<ResourceResult>>,
    next_host_id: usize,
    next_service_id: usize,
}

impl Deployment {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(non_snake_case)]
    pub fn Localhost(&mut self) -> Arc<LocalhostHost> {
        self.add_host(LocalhostHost::new)
    }

    #[allow(non_snake_case)]
    pub fn CustomService(
        &mut self,
        on: Arc<dyn Host>,
        external_ports: Vec<u16>,
    ) -> Arc<RwLock<CustomService>> {
        self.add_service(|id| CustomService::new(id, on, external_ports))
    }

    pub async fn deploy(&mut self) -> Result<()> {
        self.services.retain(|weak| weak.strong_count() > 0);

        progress::ProgressTracker::with_group("deploy", None, || async {
            let mut resource_batch = super::ResourceBatch::new();

            for service in self.services.iter().filter_map(Weak::upgrade) {
                service.read().await.collect_resources(&mut resource_batch);
            }

            for host in self.hosts.iter() {
                host.collect_resources(&mut resource_batch);
            }

            let resource_result = Arc::new(
                progress::ProgressTracker::with_group("provision", None, || async {
                    resource_batch
                        .provision(&mut self.resource_pool, self.last_resource_result.clone())
                        .await
                })
                .await?,
            );
            self.last_resource_result = Some(resource_result.clone());

            for host in self.hosts.iter() {
                host.provision(&resource_result);
            }

            progress::ProgressTracker::with_group("deploy", None, || {
                let services_future = self
                    .services
                    .iter()
                    .filter_map(Weak::upgrade)
                    .map(|service: Arc<RwLock<dyn Service>>| {
                        let resource_result = &resource_result;
                        async move { service.write().await.deploy(resource_result).await }
                    })
                    .collect::<Vec<_>>();

                futures::stream::iter(services_future)
                    .buffer_unordered(16)
                    .try_fold((), |_, _| async { Ok(()) })
            })
            .await?;

            progress::ProgressTracker::with_group("ready", None, || {
                let all_services_ready = self.services.iter().filter_map(Weak::upgrade).map(
                    |service: Arc<RwLock<dyn Service>>| async move {
                        service.write().await.ready().await?;
                        Ok(()) as Result<()>
                    },
                );

                futures::future::try_join_all(all_services_ready)
            })
            .await?;

            Ok(())
        })
        .await
    }

    pub async fn start(&mut self) -> Result<()> {
        self.services.retain(|weak| weak.strong_count() > 0);

        progress::ProgressTracker::with_group("start", None, || {
            let all_services_start = self.services.iter().filter_map(Weak::upgrade).map(
                |service: Arc<RwLock<dyn Service>>| async move {
                    service.write().await.start().await?;
                    Ok(()) as Result<()>
                },
            );

            futures::future::try_join_all(all_services_start)
        })
        .await?;
        Ok(())
    }
}

impl Deployment {
    pub fn add_host<T: Host + 'static, F: FnOnce(usize) -> T>(&mut self, host: F) -> Arc<T> {
        let arc = Arc::new(host(self.next_host_id));
        self.next_host_id += 1;

        self.hosts.push(arc.clone());
        arc
    }

    pub fn add_service<T: Service + 'static>(
        &mut self,
        service: impl ServiceBuilder<Service = T>,
    ) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(service.build(self.next_service_id)));
        self.next_service_id += 1;

        let dyn_arc: Arc<RwLock<dyn Service>> = arc.clone();
        self.services.push(Arc::downgrade(&dyn_arc));
        arc
    }
}

/// Buildstructor methods.
#[buildstructor::buildstructor]
impl Deployment {
    #[allow(clippy::too_many_arguments)]
    #[builder(entry = "GcpComputeEngineHost", exit = "add")]
    pub fn add_gcp_compute_engine_host(
        &mut self,
        project: String,
        machine_type: String,
        image: String,
        region: String,
        network: Arc<RwLock<GcpNetwork>>,
        user: Option<String>,
        startup_script: Option<String>,
    ) -> Arc<GcpComputeEngineHost> {
        self.add_host(|id| {
            GcpComputeEngineHost::new(
                id,
                project,
                machine_type,
                image,
                region,
                network,
                user,
                startup_script,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[builder(entry = "AzureHost", exit = "add")]
    pub fn add_azure_host(
        &mut self,
        project: String,
        os_type: String, // linux or windows
        machine_size: String,
        image: Option<HashMap<String, String>>,
        region: String,
        user: Option<String>,
    ) -> Arc<AzureHost> {
        self.add_host(|id| AzureHost::new(id, project, os_type, machine_size, image, region, user))
    }
}
