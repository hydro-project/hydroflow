use std::sync::Arc;

use futures::SinkExt;
use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::{Deployment, Host};
use hydroflow_plus::deploy::TrybuildHost;
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<dyn Host>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let (create_host, rustflags): (HostCreator, &'static str) = if host_arg == *"gcp" {
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
            "-C opt-level=3 -C codegen-units=1 -C strip=none -C debuginfo=2 -C lto=off",
        )
    } else {
        let localhost = deployment.Localhost();
        (
            Box::new(move |_| -> Arc<dyn Host> { localhost.clone() }),
            "",
        )
    };

    let builder = hydroflow_plus::FlowBuilder::new();
    let external = builder.external_process();
    let p1 = builder.process();
    let p2 = builder.process();
    let external_port =
        hydroflow_plus_test::distributed::first_ten::first_ten_distributed(&external, &p1, &p2);
    let nodes = builder
        .with_process(
            &p1,
            TrybuildHost::new(create_host(&mut deployment)).rustflags(rustflags),
        )
        .with_process(
            &p2,
            TrybuildHost::new(create_host(&mut deployment)).rustflags(rustflags),
        )
        .with_external(&external, deployment.Localhost())
        .deploy(&mut deployment);

    deployment.deploy().await.unwrap();

    let mut external_port = nodes.connect_sink_bincode(external_port).await;

    deployment.start().await.unwrap();

    println!("Enter characters and press enter to send them over the network (ctrl-d to stop):");
    loop {
        let mut in_line = String::new();
        if std::io::stdin().read_line(&mut in_line).unwrap() == 0 {
            break;
        }

        external_port.send(in_line).await.unwrap();
    }

    deployment.stop().await.unwrap();
}
