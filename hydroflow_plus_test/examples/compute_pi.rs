use std::rc::Rc;
use std::sync::Arc;

use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus_cli_integration::{DeployClusterSpec, DeployProcessSpec};
use stageleft::RuntimeData;
use tokio::sync::RwLock;

type HostCreator = Rc<dyn Fn(&mut Deployment) -> Arc<dyn Host>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let (create_host, profile): (HostCreator, &'static str) = if host_arg == *"gcp" {
        let project = std::env::args().nth(2).unwrap();
        let network = Arc::new(RwLock::new(GcpNetwork::new(&project, None)));

        (
            Rc::new(move |deployment| -> Arc<dyn Host> {
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
            Rc::new(move |_| -> Arc<dyn Host> { localhost.clone() }),
            "dev",
        )
    };

    let create_host_clone = create_host.clone();

    let builder = hydroflow_plus::FlowBuilder::new();
    hydroflow_plus_test::cluster::compute_pi::compute_pi(
        &builder,
        &DeployProcessSpec::new(move |deployment| {
            let host = create_host(deployment);
            deployment.add_service(
                HydroflowCrate::new(".", host.clone())
                    .bin("compute_pi")
                    .profile(profile)
                    .display_name("leader"),
            )
        }),
        &DeployClusterSpec::new(move |deployment| {
            (0..8)
                .map(|idx| {
                    let host = create_host_clone(deployment);
                    deployment.add_service(
                        HydroflowCrate::new(".", host.clone())
                            .bin("compute_pi")
                            .profile(profile)
                            .display_name(format!("cluster/{}", idx)),
                    )
                })
                .collect()
        }),
        RuntimeData::new("FAKE"),
    );

    let _nodes = builder.with_default_optimize().deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
