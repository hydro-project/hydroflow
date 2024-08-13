use std::sync::Arc;

use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus_cli_integration::{DeployClusterSpec, DeployProcessSpec};
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<dyn Host>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let (create_host, profile): (HostCreator, &'static str) = if host_arg == *"gcp" {
        let project = std::env::args().nth(2).unwrap();
        let network = Arc::new(RwLock::new(GcpNetwork::new(&project, None)));

        (
            Box::new(move |deployment| -> Arc<dyn Host> {
                deployment
                    .GcpComputeEngineHost()
                    .project(&project)
                    .machine_type("e2-micro")
                    .image("debian-cloud/debian-11")
                    .region("us-west1-a")
                    .network(network.clone())
                    .add()
            }),
            "release",
        )
    } else {
        let localhost = deployment.Localhost();
        (
            Box::new(move |_| -> Arc<dyn Host> { localhost.clone() }),
            "dev",
        )
    };

    let builder = hydroflow_plus::FlowBuilder::new();
    hydroflow_plus_test::cluster::map_reduce::map_reduce(
        &builder,
        DeployProcessSpec::new({
            let host = create_host(&mut deployment);
            HydroflowCrate::new(".", host.clone())
                .bin("map_reduce")
                .profile(profile)
                .display_name("leader")
        }),
        DeployClusterSpec::new({
            (0..2)
                .map(|idx| {
                    let host = create_host(&mut deployment);
                    HydroflowCrate::new(".", host.clone())
                        .bin("map_reduce")
                        .profile(profile)
                        .display_name(format!("cluster/{}", idx))
                })
                .collect()
        }),
    );
    let _nodes = builder.with_default_optimize().deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
