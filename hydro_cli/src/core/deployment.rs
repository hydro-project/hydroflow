use super::ResourcePool;
use super::Service;

use super::Host;

use anyhow::Result;
use tokio::sync::RwLock;

use std::sync::Arc;

#[derive(Default)]
pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
    pub services: Vec<Arc<RwLock<dyn Service>>>,
    pub resource_pool: ResourcePool,
}

impl Deployment {
    pub async fn deploy(&mut self) -> Result<()> {
        let mut resource_batch = super::ResourceBatch::new();
        for service in self.services.iter_mut() {
            service.write().await.collect_resources(&mut resource_batch);
        }

        for host in self.hosts.iter_mut() {
            host.write().await.collect_resources(&mut resource_batch);
        }

        let result = Arc::new(resource_batch.provision(&mut self.resource_pool).await?);

        let hosts_provisioned =
            self.hosts
                .iter_mut()
                .map(|host: &mut Arc<RwLock<dyn Host>>| async {
                    host.write().await.provision(&result).await;
                });
        futures::future::join_all(hosts_provisioned).await;
        println!("[hydro] provisioned resources");

        let services_future =
            self.services
                .iter_mut()
                .map(|service: &mut Arc<RwLock<dyn Service>>| async {
                    service.write().await.deploy(&result).await;
                });

        futures::future::join_all(services_future).await;
        println!("[hydro] deployed services");

        let all_services_ready =
            self.services
                .iter()
                .map(|service: &Arc<RwLock<dyn Service>>| async {
                    service.write().await.ready().await?;
                    Ok(()) as Result<()>
                });

        futures::future::try_join_all(all_services_ready).await?;
        println!("[hydro] services ready");

        Ok(())
    }

    pub async fn start(&mut self) {
        let all_services_start =
            self.services
                .iter()
                .map(|service: &Arc<RwLock<dyn Service>>| async {
                    service.write().await.start().await;
                });

        futures::future::join_all(all_services_start).await;
    }

    pub fn add_host<T: Host + 'static, F: FnOnce(usize) -> T>(
        &mut self,
        host: F,
    ) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(host(self.hosts.len())));
        self.hosts.push(arc.clone());
        arc
    }

    pub fn add_service<T: Service + 'static>(&mut self, service: T) -> Arc<RwLock<T>> {
        let arc = Arc::new(RwLock::new(service));
        self.services.push(arc.clone());
        arc
    }
}
