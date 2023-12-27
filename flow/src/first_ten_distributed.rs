use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten_distributed<'a, D: Deploy<'a>>(
    graph: &'a GraphBuilder<'a, D>,
    node_builder: &impl NodeBuilder<'a, D>,
) -> D::Node {
    let node = graph.node(node_builder);
    let second_node = graph.node(node_builder);

    let numbers = node.source_iter(q!(0..10));
    numbers
        .send_bincode(&second_node)
        .for_each(q!(|n| println!("{}", n)));

    second_node
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn first_ten_distributed_runtime<'a>(
    graph: &'a GraphBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = first_ten_distributed(graph, &cli);
    graph.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus::futures::StreamExt;
    use hydroflow_plus_cli_integration::{CLIDeployNodeBuilder, DeployCrateWrapper};

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::GraphBuilder::new();
        let second_node = super::first_ten_distributed(
            &builder,
            &CLIDeployNodeBuilder::new(|| {
                deployment.add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("first_ten_distributed")
                        .profile("dev"),
                )
            }),
        );

        deployment.deploy().await.unwrap();

        let second_node_stdout = second_node.stdout().await;

        deployment.start().await.unwrap();

        assert_eq!(
            second_node_stdout.take(10).collect::<Vec<_>>().await,
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
    }
}
