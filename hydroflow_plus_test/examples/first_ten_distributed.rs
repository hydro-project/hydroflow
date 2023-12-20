use std::sync::Arc;

use hydro_deploy::gcp::GCPNetwork;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus_cli_integration::CLIDeployNodeBuilder;
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<RwLock<dyn Host>>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
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
        let localhost = deployment.Localhost();
        (
            Box::new(move |_| -> Arc<RwLock<dyn Host>> { localhost.clone() }),
            "dev",
        )
    };

    let builder = hydroflow_plus::HfBuilder::new();
    hydroflow_plus_test::first_ten::first_ten_distributed(
        &builder,
        &CLIDeployNodeBuilder::new(|id| {
            let host = create_host(&mut deployment);
            deployment.add_service(
                HydroflowCrate::new(".", host.clone())
                    .bin("first_ten_distributed")
                    .profile(profile)
                    .args(vec![id.to_string()]),
            )
        }),
    );
    builder.wire();

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap()
}
