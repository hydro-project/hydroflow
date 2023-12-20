use std::cell::RefCell;
use std::sync::Arc;

use hydro_deploy::gcp::GCPNetwork;
use hydro_deploy::{Deployment, Host};
use hydroflow_plus_cli_integration::{CLIDeployClusterBuilder, CLIDeployNodeBuilder};
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<RwLock<dyn Host>>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    let deployment = RefCell::new(Deployment::new());
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let (create_host, profile): (HostCreator, Option<String>) = if host_arg == *"gcp" {
        let project = std::env::args().nth(2).unwrap();
        let network = Arc::new(RwLock::new(GCPNetwork::new(&project, None)));

        (
            Box::new(move |deployment| -> Arc<RwLock<dyn Host>> {
                deployment.GCPComputeEngineHost(
                    &project,
                    "e2-micro",
                    "debian-cloud/debian-11",
                    "us-west1-a",
                    network.clone(),
                    None,
                )
            }),
            None,
        )
    } else {
        let localhost = deployment.borrow_mut().Localhost();
        (
            Box::new(move |_| -> Arc<RwLock<dyn Host>> { localhost.clone() }),
            Some("dev".to_string()),
        )
    };

    let builder = hydroflow_plus::HfBuilder::new();
    hydroflow_plus_test::cluster::map_reduce(
        &builder,
        &CLIDeployNodeBuilder::new(|id| {
            let mut deployment = deployment.borrow_mut();
            let host = create_host(&mut deployment);
            deployment.HydroflowCrate(
                ".",
                host.clone(),
                Some("map_reduce".into()),
                None,
                profile.clone(),
                None,
                Some(vec![id.to_string()]),
                Some("leader".to_string()),
                vec![],
            )
        }),
        &CLIDeployClusterBuilder::new(|id| {
            let mut deployment = deployment.borrow_mut();
            (0..2)
                .map(|idx| {
                    let host = create_host(&mut deployment);
                    deployment.HydroflowCrate(
                        ".",
                        host.clone(),
                        Some("map_reduce".into()),
                        None,
                        profile.clone(),
                        None,
                        Some(vec![id.to_string()]),
                        Some(format!("cluster/{}", idx)),
                        vec![],
                    )
                })
                .collect()
        }),
    );
    builder.wire();

    let mut deployment = deployment.into_inner();

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap()
}
