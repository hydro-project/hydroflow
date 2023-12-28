use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn simple_cluster<'a, D: Deploy<'a>>(
    graph: &'a GraphBuilder<'a, D>,
    node_builder: &impl NodeBuilder<'a, D>,
    cluster_builder: &impl ClusterBuilder<'a, D>,
) -> (D::Node, D::Cluster) {
    let node = graph.node(node_builder);
    let cluster = graph.cluster(cluster_builder);

    let numbers = node.source_iter(q!(0..5));
    let ids = node.source_iter(cluster.ids()).map(q!(|&id| id));

    ids.cross_product(&numbers)
        .map(q!(|(id, n)| (id, (id, n))))
        .demux_bincode(&cluster)
        .inspect(q!(|n| println!("cluster received: {:?}", n)))
        .send_bincode_tagged(&node)
        .for_each(q!(|(id, d)| println!("node received: ({}, {:?})", id, d)));

    (node, cluster)
}

pub fn many_to_many<'a, D: Deploy<'a>>(
    graph: &'a GraphBuilder<'a, D>,
    cluster_builder: &impl ClusterBuilder<'a, D>,
) -> D::Cluster {
    let cluster = graph.cluster(cluster_builder);
    cluster
        .source_iter(q!(0..2))
        .broadcast_bincode_tagged(&cluster)
        .for_each(q!(|n| println!("cluster received: {:?}", n)));

    cluster
}

pub fn map_reduce<'a, D: Deploy<'a>>(
    graph: &'a GraphBuilder<'a, D>,
    node_builder: &impl NodeBuilder<'a, D>,
    cluster_builder: &impl ClusterBuilder<'a, D>,
) -> (D::Node, D::Cluster) {
    let node = graph.node(node_builder);
    let cluster = graph.cluster(cluster_builder);

    let words = node
        .source_iter(q!(vec!["abc", "abc", "xyz"]))
        .map(q!(|s| s.to_string()));

    let all_ids_vec = cluster.ids();
    let words_partitioned = words.enumerate().map(q!({
        let cluster_size = all_ids_vec.len();
        move |(i, w)| ((i % cluster_size) as u32, w)
    }));

    words_partitioned
        .demux_bincode(&cluster)
        .batched()
        .fold(q!(|| 0), q!(|count, string| *count += string.len()))
        .inspect(q!(|count| println!("partition count: {}", count)))
        .send_bincode_tagged(&node)
        .persist()
        .map(q!(|(_mid, count)| count))
        .fold(q!(|| 0), q!(|total, count| *total += count))
        .for_each(q!(|data| println!("total: {}", data)));

    (node, cluster)
}

use hydroflow::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn simple_cluster_runtime<'a>(
    graph: &'a GraphBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = simple_cluster(graph, &cli, &cli);
    graph.build(q!(cli.meta.subgraph_id))
}

#[stageleft::entry]
pub fn many_to_many_runtime<'a>(
    graph: &'a GraphBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = many_to_many(graph, &cli);
    graph.build(q!(cli.meta.subgraph_id))
}

#[stageleft::entry]
pub fn map_reduce_runtime<'a>(
    graph: &'a GraphBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = map_reduce(graph, &cli, &cli);
    graph.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::time::Duration;

    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow::lattices::cc_traits::Iter;
    use hydroflow_plus_cli_integration::{
        CLIDeployClusterBuilder, CLIDeployNodeBuilder, DeployCrateWrapper,
    };

    #[tokio::test]
    async fn simple_cluster() {
        let deployment = RefCell::new(Deployment::new());
        let localhost = deployment.borrow_mut().Localhost();

        let builder = hydroflow_plus::GraphBuilder::new();
        let (node, cluster) = super::simple_cluster(
            &builder,
            &CLIDeployNodeBuilder::new(|| {
                deployment.borrow_mut().add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("simple_cluster")
                        .profile("dev"),
                )
            }),
            &CLIDeployClusterBuilder::new(|| {
                (0..2)
                    .map(|_| {
                        deployment.borrow_mut().add_service(
                            HydroflowCrate::new(".", localhost.clone())
                                .bin("simple_cluster")
                                .profile("dev"),
                        )
                    })
                    .collect()
            }),
        );

        let mut deployment = deployment.into_inner();

        deployment.deploy().await.unwrap();

        let node_stdout = node.stdout().await;
        let cluster_stdouts =
            futures::future::join_all(cluster.nodes.iter().map(|node| node.stdout())).await;

        deployment.start().await.unwrap();

        for (i, stdout) in cluster_stdouts.into_iter().enumerate() {
            for j in 0..5 {
                assert_eq!(
                    tokio::time::timeout(Duration::from_secs(30), stdout.recv())
                        .await
                        .unwrap()
                        .unwrap(),
                    format!("cluster received: ({}, {})", i, j)
                );
            }
        }

        let mut node_outs = vec![];
        for _i in 0..10 {
            node_outs.push(
                tokio::time::timeout(Duration::from_secs(30), node_stdout.recv())
                    .await
                    .unwrap()
                    .unwrap(),
            );
        }
        node_outs.sort();

        for (i, n) in node_outs.into_iter().enumerate() {
            assert_eq!(
                n,
                format!("node received: ({}, ({}, {}))", i / 5, i / 5, i % 5)
            );
        }
    }

    #[tokio::test]
    async fn many_to_many() {
        let deployment = RefCell::new(Deployment::new());
        let localhost = deployment.borrow_mut().Localhost();

        let builder = hydroflow_plus::GraphBuilder::new();
        let cluster = super::many_to_many(
            &builder,
            &CLIDeployClusterBuilder::new(|| {
                (0..2)
                    .map(|_| {
                        deployment.borrow_mut().add_service(
                            HydroflowCrate::new(".", localhost.clone())
                                .bin("many_to_many")
                                .profile("dev"),
                        )
                    })
                    .collect()
            }),
        );

        let mut deployment = deployment.into_inner();

        deployment.deploy().await.unwrap();

        let cluster_stdouts =
            futures::future::join_all(cluster.nodes.iter().map(|node| node.stdout())).await;

        deployment.start().await.unwrap();

        for node_stdout in cluster_stdouts {
            let mut node_outs = vec![];
            for _i in 0..4 {
                node_outs.push(
                    tokio::time::timeout(Duration::from_secs(30), node_stdout.recv())
                        .await
                        .unwrap()
                        .unwrap(),
                );
            }
            node_outs.sort();

            let mut node_outs = node_outs.into_iter();

            for sender in 0..2 {
                for value in 0..2 {
                    assert_eq!(
                        node_outs.next().unwrap(),
                        format!("cluster received: ({}, {})", sender, value)
                    );
                }
            }
        }
    }
}
