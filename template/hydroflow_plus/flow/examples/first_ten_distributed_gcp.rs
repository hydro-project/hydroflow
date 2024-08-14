use std::sync::Arc;

use hydro_deploy::{gcp::GcpNetwork, Deployment, HydroflowCrate};
use hydroflow_plus_cli_integration::DeployProcessSpec;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let gcp_project = std::env::args()
        .nth(1)
        .expect("Expected GCP project as first argument");

    let mut deployment = Deployment::new();
    let vpc = Arc::new(RwLock::new(GcpNetwork::new(&gcp_project, None)));

    let flow = hydroflow_plus::FlowBuilder::new();
    flow::first_ten_distributed::first_ten_distributed(
        &flow,
        DeployProcessSpec::new({
            let host = deployment
                .GcpComputeEngineHost()
                .project(gcp_project.clone())
                .machine_type("e2-micro")
                .image("debian-cloud/debian-11")
                .region("us-west1-a")
                .network(vpc.clone())
                .add();

            HydroflowCrate::new(".", host).bin("first_ten_distributed")
        }),
    );

    let _nodes = flow.with_default_optimize().deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
