use std::sync::Arc;

use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::hydroflow_crate::perf_options::PerfOptions;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus_cli_integration::{DeployClusterSpec, DeployProcessSpec, TrybuildHost};
use stageleft::RuntimeData;
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<dyn Host>>;

// TODO(shadaj): currently broken due to `profile` profile not being found

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
                let startup_script = "sudo sh -c 'apt update && apt install -y linux-perf binutils && echo -1 > /proc/sys/kernel/perf_event_paranoid && echo 0 > /proc/sys/kernel/kptr_restrict'";
                deployment
                    .GcpComputeEngineHost()
                    .project(&project)
                    .machine_type("e2-micro")
                    .image("debian-cloud/debian-11")
                    .region("us-west1-a")
                    .network(network.clone())
                    .startup_script(startup_script)
                    .add()
            }),
            "profile",
        )
    } else {
        let localhost = deployment.Localhost();
        (
            Box::new(move |_| -> Arc<dyn Host> { localhost.clone() }),
            "profile",
        )
    };

    let builder = hydroflow_plus::FlowBuilder::new();
    let (cluster, leader) =
        hydroflow_plus_test::cluster::compute_pi::compute_pi(&builder, 8192);

    // Uncomment below, change .bin("counter_compute_pi") in order to track cardinality per operation
    // let runtime_context = builder.runtime_context();
    // dbg!(builder.with_default_optimize()
    //     .optimize_with(|ir| profiling(ir, runtime_context, RuntimeData::new("FAKE"), RuntimeData::new("FAKE")))
    //     .ir());

    let _nodes = builder
        .with_default_optimize()
        .with_process(
            &leader,
            TrybuildHost::new(create_host(&mut deployment))
                .profile(profile)
                .perf(
                    PerfOptions::builder()
                        .perf_outfile("leader.perf")
                        .fold_outfile("leader.data.folded")
                        .flamegraph_outfile("leader.svg")
                        .frequency(5)
                        .build()
                )
                .display_name("leader"),
        )
        .with_cluster(
            &cluster,
            (0..8).map(|idx| {
                TrybuildHost::new(create_host(&mut deployment))
                    .profile(profile)
                    .perf(
                        PerfOptions::builder()
                            .perf_outfile(format!("cluster{}.leader.perf", idx))
                            .fold_outfile(format!("cluster{}.data.folded", idx))
                            .flamegraph_outfile(format!("cluster{}.svg", idx))
                            .frequency(5)
                            .build()
                    )
                    .display_name(format!("cluster/{}", idx))
            }).collect::<Vec<_>>(),
        )
        .deploy(&mut deployment);
    deployment.run_ctrl_c().await.unwrap();
}
