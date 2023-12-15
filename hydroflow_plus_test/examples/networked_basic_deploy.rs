use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
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

    let sender_to_zero = deployment.CustomService(localhost.clone(), vec![]);
    let mut port_to_zero = sender_to_zero.read().await.create_port(&sender_to_zero);

    let mut node_zero_input = node_zero
        .underlying
        .read()
        .await
        .get_port("node_zero_input".to_string(), &node_zero.underlying);

    port_to_zero.send_to(&mut node_zero_input);

    deployment.deploy().await.unwrap();

    let mut conn_to_zero = port_to_zero.connect().await.into_sink();

    deployment.start().await.unwrap();

    for line in std::io::stdin().lines() {
        conn_to_zero.send(line.unwrap().into()).await.unwrap();
    }
}
