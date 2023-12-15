use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use tokio::sync::RwLock;

use super::{
    progress, CustomService, Host, HydroflowCrate, LocalhostHost, ResourcePool, ResourceResult,
    Service,
};

#[derive(Default)]
pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
    pub services: Vec<Arc<RwLock<dyn Service>>>,
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
    pub fn CustomService(
        &mut self,
        on: Arc<RwLock<dyn Host>>,
        external_ports: Vec<u16>,
    ) -> Arc<RwLock<CustomService>> {
        self.add_service(|id| CustomService::new(id, on, external_ports))
    }

    #[allow(non_snake_case, clippy::too_many_arguments)]
    pub fn HydroflowCrate(
        &mut self,
        src: impl Into<PathBuf>,
        on: Arc<RwLock<dyn Host>>,
        bin: Option<String>,
        example: Option<String>,
        profile: Option<String>,
        features: Option<Vec<String>>,
        args: Option<Vec<String>>,
        display_id: Option<String>,
        external_ports: Vec<u16>,
    ) -> Arc<RwLock<HydroflowCrate>> {
        self.add_service(|id| {
            crate::core::HydroflowCrate::new(
                id,
                src.into(),
                on,
                bin,
                example,
                profile,
                features,
                args,
                display_id,
                external_ports,
            )
        })
    }

    pub async fn deploy(&mut self) -> Result<()> {
        progress::ProgressTracker::with_group("deploy", None, || async {
            let mut resource_batch = super::ResourceBatch::new();

            for service in self.services.iter_mut() {
                service.write().await.collect_resources(&mut resource_batch);
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
                    .map(|service: &mut Arc<RwLock<dyn Service>>| async {
                        service.write().await.deploy(&result).await
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
                        .map(|service: &Arc<RwLock<dyn Service>>| async {
                            service.write().await.ready().await?;
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
        progress::ProgressTracker::with_group("start", None, || {
            let all_services_start =
                self.services
                    .iter()
                    .map(|service: &Arc<RwLock<dyn Service>>| async {
                        service.write().await.start().await?;
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
        service: impl FnOnce(usize) -> T,
    ) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(service(self.next_service_id)));
        self.next_service_id += 1;

        let dyn_arc: Arc<RwLock<dyn Service>> = arc.clone();
        self.services.push(dyn_arc);
        arc
    }
}
