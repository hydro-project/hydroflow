use std::cell::RefCell;
use std::sync::Arc;

use hydro_deploy::gcp::GCPNetwork;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus_cli_integration::{DeployClusterSpec, DeployProcessSpec};
use stageleft::RuntimeData;
use tokio::sync::RwLock;
use hydroflow_plus::profiler::profiling;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<RwLock<dyn Host>>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
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
            "profile",
        )
    } else {
        let localhost = deployment.borrow_mut().Localhost();
        (
            Box::new(move |_| -> Arc<RwLock<dyn Host>> { localhost.clone() }),
            "profile",
        )
    };

    let builder = hydroflow_plus::FlowBuilder::new();
    hydroflow_plus_test::cluster::compute_pi(
        &builder,
        &DeployProcessSpec::new(|| {
            let mut deployment = deployment.borrow_mut();
            let host = create_host(&mut deployment);
            deployment.add_service(
                HydroflowCrate::new(".", host.clone())
                    .bin("compute_pi")
                    .profile(profile)
                    .perf(true)
                    .display_name("leader"),
            )
        }),
        &DeployClusterSpec::new(|| {
            let mut deployment = deployment.borrow_mut();
            (0..8)
                .map(|idx| {
                    let host = create_host(&mut deployment);
                    deployment.add_service(
                        HydroflowCrate::new(".", host.clone())
                            .bin("compute_pi")
                            .profile(profile)
                            .perf(true)
                            .display_name(format!("cluster/{}", idx)),
                    )
                })
                .collect()
        }),
        RuntimeData::new("FAKE"),
    );

    // Uncomment below, change .bin("counter_compute_pi") in order to track cardinality per operation
    // let runtime_context = builder.runtime_context();
    // dbg!(builder.extract()
    //     .with_default_optimize()
    //     .optimize_with(|ir| profiling(ir, runtime_context, RuntimeData::new("FAKE"), RuntimeData::new("FAKE")))
    //     .ir());

    let mut deployment = deployment.into_inner();

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap()
}
