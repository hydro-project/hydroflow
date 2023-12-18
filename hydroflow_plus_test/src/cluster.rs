use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn simple_cluster<'a, D: HfNetworkedDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    // node_builder: &mut impl HfNodeBuilder<'a, D>,
    cluster_builder: &mut impl HfClusterBuilder<'a, D>,
) -> D::Cluster {
    // let node = graph.node(node_builder);
    let cluster = graph.cluster(cluster_builder);

    let numbers = cluster.source_iter(q!(0..10));
    numbers.for_each(q!(|n| println!("{}", n)));

    let ids = cluster.source_iter(cluster.ids());
    ids.for_each(q!(|n| println!("id: {}", n)));

    cluster
}

use hydroflow::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, CLIRuntimeClusterBuilder, HydroflowPlusMeta};

#[stageleft::entry]
pub fn simple_cluster_runtime<'a>(
    graph: &'a HfBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = simple_cluster(graph, &mut CLIRuntimeClusterBuilder::new(cli));
    graph.build(node_id)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hydro_cli::core::Deployment;
    use hydroflow_plus_cli_integration::{CLIDeployClusterBuilder, CLIDeployNodeBuilder};

    #[tokio::test]
    async fn simple_cluster() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::HfBuilder::new();
        let second_node = super::simple_cluster(
            &builder,
            &mut CLIDeployClusterBuilder::new(|id| {
                vec![deployment.HydroflowCrate(
                    ".",
                    localhost.clone(),
                    Some("cluster".into()),
                    None,
                    Some("dev".into()),
                    None,
                    Some(vec![id.to_string()]),
                    None,
                    vec![],
                )]
            }),
        );
        builder.wire();

        deployment.deploy().await.unwrap();

        // let second_node_stdout = second_node.stdout().await;

        deployment.start().await.unwrap();

        todo!()

        // for i in 0..10 {
        //     assert_eq!(
        //         tokio::time::timeout(Duration::from_secs(1), second_node_stdout.recv())
        //             .await
        //             .unwrap()
        //             .unwrap(),
        //         i.to_string()
        //     );
        // }
    }
}
