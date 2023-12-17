use hydroflow::bytes::BytesMut;
use hydroflow::util::cli::HydroCLI;
use hydroflow_plus::node::{HfNetworkedDeploy, HfNode, HfNodeBuilder};
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::HfBuilder;
use hydroflow_plus_cli_integration::{CLIRuntime, CLIRuntimeNodeBuilder};
use stageleft::{q, Quoted, RuntimeData};

pub fn networked_basic<'a, D: HfNetworkedDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    node_builder: &mut impl HfNodeBuilder<'a, D>,
) -> (D::Port, D::Node, D::Node) {
    let node_zero = graph.node(node_builder);
    let node_one = graph.node(node_builder);

    let (source_zero_port, source_zero) = node_zero.source_external();

    source_zero
        .map(q!(|v| v.unwrap().freeze()))
        .send_to(&node_one)
        .for_each(q!(|v: Result<BytesMut, _>| {
            println!(
                "node one received: {:?}",
                std::str::from_utf8(&v.unwrap()).unwrap()
            );
        }));

    (source_zero_port, node_zero, node_one)
}

#[stageleft::entry]
pub fn networked_basic_runtime<'a>(
    graph: &'a HfBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = networked_basic(graph, &mut CLIRuntimeNodeBuilder::new(cli));
    graph.build(node_id)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hydro_cli::core::Deployment;
    use hydroflow::futures::SinkExt;
    use hydroflow::util::cli::ConnectedSink;
    use hydroflow_plus_cli_integration::CLIDeployNodeBuilder;

    #[tokio::test]
    async fn networked_basic() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::HfBuilder::new();
        let (source_zero_port, _, node_one) = super::networked_basic(
            &builder,
            &mut CLIDeployNodeBuilder::new(|id| {
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

        let port_to_zero = source_zero_port
            .create_sender(&mut deployment, &localhost)
            .await;

        deployment.deploy().await.unwrap();

        let mut conn_to_zero = port_to_zero.connect().await.into_sink();
        let node_one_stdout = node_one.stdout().await;

        deployment.start().await.unwrap();

        conn_to_zero.send("hello world!".into()).await.unwrap();

        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), node_one_stdout.recv())
                .await
                .unwrap()
                .unwrap(),
            "node one received: \"hello world!\""
        );
    }
}
