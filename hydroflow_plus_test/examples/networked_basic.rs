use hydro_deploy::Deployment;
use hydroflow::futures::SinkExt;
use hydroflow::util::cli::ConnectedSink;
use hydroflow_plus_cli_integration::CLIDeployNodeBuilder;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let builder = hydroflow_plus::HfBuilder::new();
    let (source_zero_port, _, _) = hydroflow_plus_test::networked::networked_basic(
        &builder,
        &CLIDeployNodeBuilder::new(|id| {
            deployment.HydroflowCrate(
                ".",
                localhost.clone(),
                Some("networked_basic".into()),
                None,
                Some("dev".into()),
                None,
                Some(vec![id.to_string()]),
                None,
                vec![],
            )
        }),
    );
    builder.wire();

    let port_to_zero = source_zero_port
        .create_sender(&mut deployment, &localhost)
        .await;

    deployment.deploy().await.unwrap();

    let mut conn_to_zero = port_to_zero.connect().await.into_sink();

    deployment.start().await.unwrap();

    for line in std::io::stdin().lines() {
        conn_to_zero.send(line.unwrap().into()).await.unwrap();
    }
}
