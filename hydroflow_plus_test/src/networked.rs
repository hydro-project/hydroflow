use hydroflow::bytes::BytesMut;
use hydroflow::util::cli::HydroCLI;
use hydroflow_plus::node::{CLIRuntime, CLIRuntimeNodeBuilder, HFDeploy, HFNodeBuilder};
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::HfBuilder;
use stageleft::{q, Quoted, RuntimeData};

pub fn networked_basic<'a, D: HFDeploy<'a>>(
    graph: &'a HfBuilder<'a>,
    node_builder: &mut D::NodeBuilder,
) -> (D::Node, D::Node) {
    let node_zero = node_builder.build(graph);
    let node_one = node_builder.build(graph);

    let source_zero = graph.source_port(&node_zero, "node_zero_input");

    source_zero
        .map(q!(|v| v.unwrap().freeze()))
        .send_to(&node_one)
        .for_each(q!(|v: Result<BytesMut, _>| {
            println!(
                "node one received: {:?}",
                std::str::from_utf8(&v.unwrap()).unwrap()
            );
        }));

    (node_zero, node_one)
}

#[stageleft::entry]
pub fn networked_basic_runtime<'a>(
    graph: &'a HfBuilder<'a>,
    cli: RuntimeData<&'a HydroCLI>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let mut node_zero = CLIRuntimeNodeBuilder::new(cli);
    let _ = networked_basic::<CLIRuntime>(graph, &mut node_zero);
    graph.build(node_id)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hydro_cli::core::Deployment;
    use hydroflow::futures::SinkExt;
    use hydroflow::util::cli::ConnectedSink;
    use hydroflow_plus_cli_integration::{CLIDeploy, CLIDeployNodeBuilder};

    #[tokio::test]
    async fn networked_basic() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::HfBuilder::new();
        let (node_zero, node_one) = super::networked_basic::<CLIDeploy>(
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
