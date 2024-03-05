use std::sync::{Arc, Weak};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use tokio::sync::RwLock;

use super::gcp::GCPNetwork;
use super::{
    progress, CustomService, GCPComputeEngineHost, Host, LocalhostHost, ResourcePool,
    ResourceResult, Service,
};
use crate::ServiceBuilder;

#[derive(Default)]
pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
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
    pub fn Localhost(&mut self) -> Arc<RwLock<LocalhostHost>> {
        self.add_host(LocalhostHost::new)
    }

    #[allow(non_snake_case)]
    pub fn GCPComputeEngineHost(
        &mut self,
        project: impl Into<String>,
        machine_type: impl Into<String>,
        image: impl Into<String>,
        region: impl Into<String>,
        network: Arc<RwLock<GCPNetwork>>,
        user: Option<String>,
    ) -> Arc<RwLock<GCPComputeEngineHost>> {
        self.add_host(|id| {
            GCPComputeEngineHost::new(id, project, machine_type, image, region, network, user)
        })
    }

    #[allow(non_snake_case)]
    pub fn CustomService(
        &mut self,
        on: Arc<RwLock<dyn Host>>,
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

            for service in self.services.iter_mut() {
                service
                    .upgrade()
                    .unwrap()
                    .write()
                    .await
                    .collect_resources(&mut resource_batch);
            }

            for host in self.hosts.iter_mut() {
                host.write().await.collect_resources(&mut resource_batch);
            }

            let result = Arc::new(
                progress::ProgressTracker::with_group("provision", None, || async {
                    resource_batch
                        .provision(&mut self.resource_pool, self.last_resource_result.clone())
                        .await
                })
                .await?,
            );
            self.last_resource_result = Some(result.clone());

            progress::ProgressTracker::with_group("provision", None, || {
                let hosts_provisioned =
                    self.hosts
                        .iter_mut()
                        .map(|host: &mut Arc<RwLock<dyn Host>>| async {
                            host.write().await.provision(&result).await;
                        });
                futures::future::join_all(hosts_provisioned)
            })
            .await;

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
                            .deploy(&result)
                            .await
                    })
                    .collect::<Vec<_>>();

                futures::stream::iter(services_future)
                    .buffer_unordered(8)
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

    pub fn add_host<T: Host + 'static, F: FnOnce(usize) -> T>(
        &mut self,
        host: F,
    ) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(host(self.next_host_id)));
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
