use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten<'a, D: HfDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    node_builder: &impl HfNodeBuilder<'a, D>,
) {
    let node = graph.node(node_builder);
    let numbers = node.source_iter(q!(0..10));
    numbers.for_each(q!(|n| println!("{}", n)));
}

#[stageleft::entry]
pub fn first_ten_runtime<'a>(
    graph: &'a HfBuilder<'a, SingleGraph>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    first_ten(graph, &());
    graph.build_single()
}

pub fn first_ten_distributed<'a, D: HfNetworkedDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    node_builder: &impl HfNodeBuilder<'a, D>,
) -> D::Node {
    let node = graph.node(node_builder);
    let second_node = graph.node(node_builder);

    let numbers = node.source_iter(q!(0..10));
    numbers
        .send_bincode(&second_node)
        .for_each(q!(|n| println!("{}", n)));

    second_node
}

use hydroflow::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn first_ten_distributed_runtime<'a>(
    graph: &'a HfBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = first_ten_distributed(graph, &cli);
    graph.build(node_id)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus_cli_integration::{CLIDeployNodeBuilder, DeployCrateWrapper};

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::HfBuilder::new();
        let second_node = super::first_ten_distributed(
            &builder,
            &CLIDeployNodeBuilder::new(|id| {
                deployment.add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("first_ten_distributed")
                        .profile("dev")
                        .args(vec![id.to_string()]),
                )
            }),
        );
        builder.wire();

        deployment.deploy().await.unwrap();

        let second_node_stdout = second_node.stdout().await;

        deployment.start().await.unwrap();

        for i in 0..10 {
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(30), second_node_stdout.recv())
                    .await
                    .unwrap()
                    .unwrap(),
                i.to_string()
            );
        }
    }
}
