use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
use hydro_cli::core::Deployment;
use hydroflow::futures::SinkExt;
use hydroflow::util::cli::{ConnectedDirect, ConnectedSink};
use hydroflow_plus_cli_integration::CLIDeployNode;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let sender = deployment.CustomService(localhost.clone(), vec![]);

    let service_zero = CLIDeployNode::new(
        0,
        deployment.HydroflowCrate(
            ".",
            localhost.clone(),
            None,
            Some("networked_basic".into()),
            Some("dev".into()),
            None,
            Some(vec!["0".into()]),
            None,
            vec![],
        ),
    );

    let service_one = CLIDeployNode::new(
        1,
        deployment.HydroflowCrate(
            ".",
            localhost.clone(),
            None,
            Some("networked_basic".into()),
            Some("dev".into()),
            None,
            Some(vec!["1".into()]),
            None,
            vec![],
        ),
    );

    let builder = hydroflow_plus::HfBuilder::new();
    hydroflow_plus_test::networked::networked_basic(&builder, &service_zero, &service_one);

    let mut sender_port = sender.read().await.create_port(&sender);

    let mut node_zero_input = service_zero
        .underlying
        .read()
        .await
        .get_port("node_zero_input".to_string(), &service_zero.underlying);

    sender_port.send_to(&mut node_zero_input);

    deployment.deploy().await.unwrap();

    let mut connection = sender_port
        .server_port()
        .await
        .instantiate()
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    deployment.start().await.unwrap();

    for line in std::io::stdin().lines() {
        connection.send(line.unwrap().into()).await.unwrap();
    }
}
