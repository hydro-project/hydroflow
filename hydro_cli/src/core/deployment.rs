use super::ResourcePool;
use super::ResourceResult;
use super::Service;

use super::Host;

use anyhow::Result;
use futures::Future;
use indicatif::MultiProgress;
use tokio::sync::RwLock;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::Weak;
use std::time::Duration;

static PROGRESS_TRACKER: OnceLock<Mutex<ProgressTracker>> = OnceLock::new();

tokio::task_local! {
    static CURRENT_GROUP: Vec<String>;
}

pub struct ProgressTracker {
    multi_progress: MultiProgress,
    current_count: usize,
}

impl ProgressTracker {
    fn new() -> ProgressTracker {
        ProgressTracker {
            multi_progress: MultiProgress::new(),
            current_count: 0,
        }
    }

    pub fn start_task(&mut self, path: Vec<String>) -> Arc<indicatif::ProgressBar> {
        self.current_count += 1;
        let pb = Arc::new(self.multi_progress.add(indicatif::ProgressBar::new(100)));
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} {wide_msg} ({elapsed} elapsed)")
                .unwrap(),
        );
        pb.set_message(path.join(" / "));
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn start_task_progress(&mut self, path: Vec<String>) -> Arc<indicatif::ProgressBar> {
        self.current_count += 1;
        let pb = Arc::new(self.multi_progress.add(indicatif::ProgressBar::new(100)));
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} {wide_msg} {bar} ({elapsed} elapsed)")
                .unwrap(),
        );
        pb.set_message(path.join(" / "));
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn end_task(&mut self, pb: Arc<indicatif::ProgressBar>) {
        self.current_count -= 1;
        self.multi_progress.remove(&pb);

        if self.current_count == 0 {
            self.multi_progress.clear().unwrap();
        }
    }
}

impl ProgressTracker {
    pub fn with_group<T, F: Future<Output = T>>(name: &str, f: F) -> impl Future<Output = T> {
        let mut cur = CURRENT_GROUP
            .try_with(|cur| cur.clone())
            .unwrap_or_default();
        cur.push(name.to_string());
        CURRENT_GROUP.scope(cur, f)
    }

    pub fn leaf<T, F: Future<Output = T>>(name: String, f: F) -> impl Future<Output = T> {
        let mut group = CURRENT_GROUP
            .try_with(|cur| cur.clone())
            .unwrap_or_default();
        group.push(name);
        let bar = {
            let mut progress_bar = PROGRESS_TRACKER
                .get_or_init(|| Mutex::new(ProgressTracker::new()))
                .lock()
                .unwrap();
            progress_bar.start_task(group)
        };

        async {
            let out = f.await;
            let mut progress_bar = PROGRESS_TRACKER
                .get_or_init(|| Mutex::new(ProgressTracker::new()))
                .lock()
                .unwrap();
            progress_bar.end_task(bar);
            out
        }
    }

    pub fn leaf_with_progress<'a, T, F: Future<Output = T>>(
        name: String,
        f: impl FnOnce(Arc<dyn Fn(u64) -> () + Send + Sync>) -> F + 'a,
    ) -> impl Future<Output = T> + 'a {
        let mut group = CURRENT_GROUP
            .try_with(|cur| cur.clone())
            .unwrap_or_default();
        group.push(name);
        let bar = {
            let mut progress_bar = PROGRESS_TRACKER
                .get_or_init(|| Mutex::new(ProgressTracker::new()))
                .lock()
                .unwrap();
            progress_bar.start_task_progress(group)
        };

        async {
            let my_bar = bar.clone();
            let out = f(Arc::new(move |progress| {
                my_bar.set_position(progress);
            }))
            .await;
            let mut progress_bar = PROGRESS_TRACKER
                .get_or_init(|| Mutex::new(ProgressTracker::new()))
                .lock()
                .unwrap();
            progress_bar.end_task(bar);
            out
        }
    }
}

#[derive(Default)]
pub struct Deployment {
    pub hosts: Vec<Arc<RwLock<dyn Host>>>,
    pub services: Vec<Weak<RwLock<dyn Service>>>,
    pub resource_pool: ResourcePool,
    last_resource_result: Option<Arc<ResourceResult>>,
}

impl Deployment {
    pub async fn deploy(&mut self) -> Result<()> {
        ProgressTracker::with_group("deploy", async {
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
                ProgressTracker::with_group("provision", async {
                    resource_batch
                        .provision(&mut self.resource_pool, self.last_resource_result.clone())
                        .await
                })
                .await?,
            );
            self.last_resource_result = Some(result.clone());

            let hosts_provisioned =
                self.hosts
                    .iter_mut()
                    .map(|host: &mut Arc<RwLock<dyn Host>>| {
                        ProgressTracker::with_group("provision", async {
                            host.write().await.provision(&result).await;
                        })
                    });
            futures::future::join_all(hosts_provisioned).await;

            let services_future =
                self.services
                    .iter_mut()
                    .map(|service: &mut Weak<RwLock<dyn Service>>| {
                        ProgressTracker::with_group("deploy", async {
                            service
                                .upgrade()
                                .unwrap()
                                .write()
                                .await
                                .deploy(&result)
                                .await;
                        })
                    });

            futures::future::join_all(services_future).await;

            let all_services_ready =
                self.services
                    .iter()
                    .map(|service: &Weak<RwLock<dyn Service>>| {
                        ProgressTracker::with_group("ready", async {
                            service.upgrade().unwrap().write().await.ready().await?;
                            Ok(()) as Result<()>
                        })
                    });

            futures::future::try_join_all(all_services_ready).await?;

            Ok(())
        })
        .await
    }

    pub async fn start(&mut self) {
        let active_services = self
            .services
            .iter()
            .filter(|service| service.upgrade().is_some())
            .cloned()
            .collect::<Vec<_>>();
        self.services = active_services;

        let all_services_start =
            self.services
                .iter()
                .map(|service: &Weak<RwLock<dyn Service>>| async {
                    service.upgrade().unwrap().write().await.start().await;
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
        let dyn_arc: Arc<RwLock<dyn Service>> = arc.clone();
        self.services.push(Arc::downgrade(&dyn_arc));
        arc
    }
}
