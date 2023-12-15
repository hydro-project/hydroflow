use hydroflow::bytes::BytesMut;
use hydroflow::util::cli::HydroCLI;
use hydroflow_plus::node::{CLIRuntimeNode, HfConnectable, HfNode};
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::HfBuilder;
use stageleft::{q, Quoted, RuntimeData};

pub fn networked_basic<'a, N0: HfNode<'a>, N1: HfNode<'a>>(
    graph: &'a HfBuilder<'a>,
    node_zero: N0,
    node_one: N1,
) where
    N0: HfConnectable<'a, N1>,
{
    let source_zero = graph.source_port(node_zero, "node_zero_input");

    source_zero
        .map(q!(|v| v.unwrap().freeze()))
        .send_to(node_one)
        .for_each(q!(|v: Result<BytesMut, _>| {
            println!(
                "node one received: {:?}",
                std::str::from_utf8(&v.unwrap()).unwrap()
            );
        }));
}

#[stageleft::entry]
pub fn networked_basic_runtime<'a>(
    graph: &'a HfBuilder<'a>,
    cli: RuntimeData<&'a HydroCLI>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let node_zero = CLIRuntimeNode::new(0, cli);
    let node_one = CLIRuntimeNode::new(1, cli);
    networked_basic(graph, &node_zero, &node_one);
    graph.build(node_id)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
    use hydro_cli::core::Deployment;
    use hydroflow::futures::SinkExt;
    use hydroflow::util::cli::ConnectedSink;
    use hydroflow_plus_cli_integration::CLIDeployNode;

    #[tokio::test]
    async fn networked_basic() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let sender_to_zero = deployment.CustomService(localhost.clone(), vec![]);

        let node_zero = CLIDeployNode::new(
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

        let node_one = CLIDeployNode::new(
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
        super::networked_basic(&builder, &node_zero, &node_one);

        let mut port_to_zero = sender_to_zero.read().await.create_port(&sender_to_zero);

        let mut node_zero_input = node_zero
            .underlying
            .read()
            .await
            .get_port("node_zero_input".to_string(), &node_zero.underlying);

        port_to_zero.send_to(&mut node_zero_input);

        deployment.deploy().await.unwrap();

        let mut conn_to_zero = port_to_zero.connect().await.into_sink();

        let service_1_stdout = node_one.underlying.read().await.stdout().await;

        deployment.start().await.unwrap();

        conn_to_zero.send("hello world!".into()).await.unwrap();

        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), service_1_stdout.recv())
                .await
                .unwrap()
                .unwrap(),
            "node one received: \"hello world!\""
        );
    }
}
