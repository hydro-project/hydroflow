use std::sync::{Arc, Weak};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use tokio::sync::RwLock;

use super::gcp::GcpNetwork;
use super::{
    progress, CustomService, GcpComputeEngineHost, Host, LocalhostHost, ResourcePool,
    ResourceResult, Service,
};
use crate::ServiceBuilder;

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

    #[allow(non_snake_case, clippy::too_many_arguments)] // TODO(mingwei)
    pub fn GcpComputeEngineHost(
        &mut self,
        project: impl Into<String>,
        machine_type: impl Into<String>,
        image: impl Into<String>,
        region: impl Into<String>,
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

    #[allow(non_snake_case)]
    pub fn CustomService(
        &mut self,
        on: Arc<dyn Host>,
        external_ports: Vec<u16>,
    ) -> Arc<RwLock<CustomService>> {
        self.add_service(|id| CustomService::new(id, on, external_ports))
    }

    pub async fn deploy(&mut self) -> Result<()> {
        progress::ProgressTracker::with_group("deploy", None, || async {
            let mut resource_batch = super::ResourceBatch::new();
            let active_services = self
                .services
                .iter()
                .filter(|service| service.upgrade().is_some())
                .cloned()
                .collect::<Vec<_>>();
            self.services = active_services;

            for service in self.services.iter() {
                service
                    .upgrade()
                    .unwrap()
                    .read()
                    .await
                    .collect_resources(&mut resource_batch);
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
                    .iter_mut()
                    .map(|service: &mut Weak<RwLock<dyn Service>>| async {
                        service
                            .upgrade()
                            .unwrap()
                            .write()
                            .await
                            .deploy(&resource_result)
                            .await
                    })
                    .collect::<Vec<_>>();

                futures::stream::iter(services_future)
                    .buffer_unordered(16)
                    .try_fold((), |_, _| async { Ok(()) })
            })
            .await?;

            progress::ProgressTracker::with_group("ready", None, || {
                let all_services_ready =
                    self.services
                        .iter()
                        .map(|service: &Weak<RwLock<dyn Service>>| async {
                            service.upgrade().unwrap().write().await.ready().await?;
                            Ok(()) as Result<()>
                        });

                futures::future::try_join_all(all_services_ready)
            })
            .await?;

            Ok(())
        })
        .await
    }

    pub async fn start(&mut self) -> Result<()> {
        let active_services = self
            .services
            .iter()
            .filter(|service| service.upgrade().is_some())
            .cloned()
            .collect::<Vec<_>>();
        self.services = active_services;

        progress::ProgressTracker::with_group("start", None, || {
            let all_services_start =
                self.services
                    .iter()
                    .map(|service: &Weak<RwLock<dyn Service>>| async {
                        service.upgrade().unwrap().write().await.start().await?;
                        Ok(()) as Result<()>
                    });

            futures::future::try_join_all(all_services_start)
        })
        .await?;
        Ok(())
    }

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
