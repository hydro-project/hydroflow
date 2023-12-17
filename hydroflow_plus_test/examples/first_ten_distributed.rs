use hydro_cli::core::Deployment;
use hydroflow_plus_cli_integration::CLIDeployNodeBuilder;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let builder = hydroflow_plus::HfBuilder::new();
    hydroflow_plus_test::first_ten::first_ten_distributed(
        &builder,
        &mut CLIDeployNodeBuilder::new(|id| {
            deployment.HydroflowCrate(
                ".",
                localhost.clone(),
                Some("first_ten_distributed".into()),
                None,
                Some("dev".into()),
                None,
                Some(vec![id.to_string()]),
                None,
                vec![],
            )
        }),
    );

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
