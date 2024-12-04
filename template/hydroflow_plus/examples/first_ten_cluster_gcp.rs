use std::sync::Arc;

use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::Deployment;
use hydroflow_plus::deploy::TrybuildHost;
use tokio::sync::RwLock;

static RELEASE_RUSTFLAGS: &str =
    "-C opt-level=3 -C codegen-units=1 -C strip=none -C debuginfo=2 -C lto=off";

#[tokio::main]
async fn main() {
    let gcp_project = std::env::args()
        .nth(1)
        .expect("Expected GCP project as first argument");

    let mut deployment = Deployment::new();
    let vpc = Arc::new(RwLock::new(GcpNetwork::new(&gcp_project, None)));

    let flow = hydroflow_plus::FlowBuilder::new();
    let leader = flow.process();
    let workers = flow.cluster();
    hydroflow_plus_template::first_ten_cluster::first_ten_cluster(&leader, &workers);

    let _nodes = flow
        .with_process(
            &leader,
            TrybuildHost::new(
                deployment
                    .GcpComputeEngineHost()
                    .project(gcp_project.clone())
                    .machine_type("e2-micro")
                    .image("debian-cloud/debian-11")
                    .region("us-west1-a")
                    .network(vpc.clone())
                    .add(),
            )
            .rustflags(RELEASE_RUSTFLAGS),
        )
        .with_process(
            &workers,
            vec![
                TrybuildHost::new(
                    deployment
                        .GcpComputeEngineHost()
                        .project(gcp_project.clone())
                        .machine_type("e2-micro")
                        .image("debian-cloud/debian-11")
                        .region("us-west1-a")
                        .network(vpc.clone())
                        .add(),
                )
                .rustflags(RELEASE_RUSTFLAGS);
                4
            ],
        )
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
