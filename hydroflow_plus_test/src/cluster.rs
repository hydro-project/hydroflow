use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn simple_cluster<'a, D: HfNetworkedDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    node_builder: &impl HfNodeBuilder<'a, D>,
    cluster_builder: &impl HfClusterBuilder<'a, D>,
) -> D::Cluster {
    let node = graph.node(node_builder);
    let cluster = graph.cluster(cluster_builder);

    let numbers = node.source_iter(q!(0..5));
    let ids = node.source_iter(cluster.ids()).map(q!(|&id| id));

    ids.cross_product(&numbers)
        .map(q!(|(id, n)| (id, (id, n))))
        .demux_bincode(&cluster)
        .for_each(q!(|n| println!("received: {:?}", n)));

    cluster
}

use hydroflow::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn simple_cluster_runtime<'a>(
    graph: &'a HfBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = simple_cluster(graph, &cli, &cli);
    graph.build(node_id)
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::time::Duration;

    use hydro_cli::core::Deployment;
    use hydroflow::lattices::cc_traits::Iter;
    use hydroflow_plus_cli_integration::{
        CLIDeployClusterBuilder, CLIDeployNodeBuilder, DeployCrateWrapper,
    };

    #[tokio::test]
    async fn simple_cluster() {
        let deployment = RefCell::new(Deployment::new());
        let localhost = deployment.borrow_mut().Localhost();

        let builder = hydroflow_plus::HfBuilder::new();
        let cluster = super::simple_cluster(
            &builder,
            &CLIDeployNodeBuilder::new(|id| {
                deployment.borrow_mut().HydroflowCrate(
                    ".",
                    localhost.clone(),
                    Some("simple_cluster".into()),
                    None,
                    Some("dev".into()),
                    None,
                    Some(vec![id.to_string()]),
                    None,
                    vec![],
                )
            }),
            &CLIDeployClusterBuilder::new(|id| {
                (0..2)
                    .map(|_| {
                        deployment.borrow_mut().HydroflowCrate(
                            ".",
                            localhost.clone(),
                            Some("simple_cluster".into()),
                            None,
                            Some("dev".into()),
                            None,
                            Some(vec![id.to_string()]),
                            None,
                            vec![],
                        )
                    })
                    .collect()
            }),
        );
        builder.wire();

        let mut deployment = deployment.into_inner();

        deployment.deploy().await.unwrap();

        let cluster_stdouts =
            futures::future::join_all(cluster.nodes.iter().map(|node| node.stdout())).await;

        deployment.start().await.unwrap();

        for (i, stdout) in cluster_stdouts.into_iter().enumerate() {
            for j in 0..5 {
                assert_eq!(
                    tokio::time::timeout(Duration::from_secs(1), stdout.recv())
                        .await
                        .unwrap()
                        .unwrap(),
                    format!("received: ({}, {})", i, j)
                );
            }
        }
    }
}
