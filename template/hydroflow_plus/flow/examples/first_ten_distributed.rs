use hydro_deploy::{Deployment, HydroflowCrate};
use hydroflow_plus_cli_integration::DeployProcessSpec;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let flow = hydroflow_plus::FlowBuilder::new();
    let (p1, p2) = flow::first_ten_distributed::first_ten_distributed(&flow);

    let nodes = flow
        .with_default_optimize()
        .with_process(
            &p1,
            DeployProcessSpec::new({
                HydroflowCrate::new(".", localhost.clone())
                    .bin("first_ten_distributed")
                    .profile("dev")
            }),
        )
        .with_process(
            &p2,
            DeployProcessSpec::new({
                HydroflowCrate::new(".", localhost.clone())
                    .bin("first_ten_distributed")
                    .profile("dev")
            }),
        )
        .deploy(&mut deployment);

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap()
}
