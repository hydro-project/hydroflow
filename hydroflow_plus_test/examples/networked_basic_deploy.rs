use hydro_cli::core::Deployment;
use hydroflow::futures::SinkExt;
use hydroflow::util::cli::ConnectedSink;
use hydroflow_plus_cli_integration::{CLIDeploy, CLIDeployNodeBuilder};

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let builder = hydroflow_plus::HfBuilder::new();
    let (node_zero, _) = hydroflow_plus_test::networked::networked_basic::<CLIDeploy>(
        &builder,
        &mut CLIDeployNodeBuilder::new(|id| {
            deployment.HydroflowCrate(
                ".",
                localhost.clone(),
                None,
                Some("networked_basic".into()),
                Some("dev".into()),
                None,
                Some(vec![id.to_string()]),
                None,
                vec![],
            )
        }),
    );

    let port_to_zero = node_zero
        .create_sender("node_zero_input", &mut deployment, &localhost)
        .await;

    deployment.deploy().await.unwrap();

    let mut conn_to_zero = port_to_zero.connect().await.into_sink();

    deployment.start().await.unwrap();

    for line in std::io::stdin().lines() {
        conn_to_zero.send(line.unwrap().into()).await.unwrap();
    }
}
