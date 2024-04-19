use std::cell::RefCell;
use std::sync::Arc;

use hydro_deploy::gcp::GCPNetwork;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus_cli_integration::{DeployClusterSpec, DeployProcessSpec};
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<RwLock<dyn Host>>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    use hydroflow_plus::properties::properties_optimize; // Import the algebraic optimization rules
    let deployment = RefCell::new(Deployment::new());
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let (create_host, profile): (HostCreator, &'static str) = if host_arg == *"gcp" {
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
            "release",
        )
    } else {
        let localhost = deployment.borrow_mut().Localhost();
        (
            Box::new(move |_| -> Arc<RwLock<dyn Host>> { localhost.clone() }),
            "dev",
        )
    };

    let builder = hydroflow_plus::FlowBuilder::new();
    let mut database = hydroflow_plus::properties::PropertyDatabase::default();
    hydroflow_plus_test::cluster::map_reduce(
        &builder,
        &mut database,
        &DeployProcessSpec::new(|| {
            let mut deployment = deployment.borrow_mut();
            let host = create_host(&mut deployment);
            deployment.add_service(
                HydroflowCrate::new(".", host.clone())
                    .bin("map_reduce")
                    .profile(profile)
                    .display_name("leader"),
            )
        }),
        &DeployClusterSpec::new(|| {
            let mut deployment = deployment.borrow_mut();
            (0..2)
                .map(|idx| {
                    let host = create_host(&mut deployment);
                    deployment.add_service(
                        HydroflowCrate::new(".", host.clone())
                            .bin("map_reduce")
                            .profile(profile)
                            .display_name(format!("cluster/{}", idx)),
                    )
                })
                .collect()
        }),
    );

    let _ = builder
        .extract()
        .optimize_with(|ir| properties_optimize(ir, &database))
        .optimize_default();

    // let mut deployment = deployment.into_inner();

    // deployment.deploy().await.unwrap();

    // deployment.start().await.unwrap();

    // tokio::signal::ctrl_c().await.unwrap()
}
