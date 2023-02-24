use std::fmt::Debug;

use super::Service;

use super::Host;

use tokio::sync::RwLock;

use std::sync::Arc;

#[derive(Clone)]
pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
    pub services: Vec<Arc<RwLock<dyn Service>>>,
}

impl Deployment {
    pub async fn deploy(&mut self) {
        let mut resource_pool = super::ResourceBatch {};
        for service in self.services.iter_mut() {
            service
                .write()
                .await
                .collect_resources(&mut resource_pool)
                .await;
        }

        let result = Arc::new(resource_pool.provision().await);

        let services_future =
            self.services
                .iter_mut()
                .map(|service: &mut Arc<RwLock<dyn Service>>| async {
                    service.write().await.deploy(&result).await;
                });

        futures::future::join_all(services_future).await;

        let all_services_ready =
            self.services
                .iter()
                .map(|service: &Arc<RwLock<dyn Service>>| async {
                    service.write().await.ready().await;
                });

        futures::future::join_all(all_services_ready).await;
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

    pub fn add_host<T: Host + 'static, F: Fn(usize) -> T>(&mut self, host: F) -> Arc<RwLock<T>> {
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

impl Debug for Deployment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deployment")
            .field(
                "hosts",
                &self
                    .hosts
                    .iter()
                    .map(|h| h.blocking_read())
                    .collect::<Vec<_>>(),
            )
            .field(
                "services",
                &self
                    .services
                    .iter()
                    .map(|h| h.blocking_read())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
