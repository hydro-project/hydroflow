use hydro_deploy::{Deployment, HydroflowCrate};
use hydroflow_plus_cli_integration::DeployProcessSpec;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let flow = hydroflow_plus::FlowBuilder::new();
    flow::first_ten_distributed::first_ten_distributed(
        &flow,
        &DeployProcessSpec::new(|| {
            deployment.add_service(
                HydroflowCrate::new(".", localhost.clone())
                    .bin("first_ten_distributed")
                    .profile("dev"),
            )
        }),
    );

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap()
}
