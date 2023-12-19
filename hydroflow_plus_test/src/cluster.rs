use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn simple_cluster<'a, D: HfNetworkedDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    node_builder: &impl HfNodeBuilder<'a, D>,
    cluster_builder: &impl HfClusterBuilder<'a, D>,
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

pub fn map_reduce<'a, D: HfNetworkedDeploy<'a>>(
    graph: &'a HfBuilder<'a, D>,
    node_builder: &impl HfNodeBuilder<'a, D>,
    cluster_builder: &impl HfClusterBuilder<'a, D>,
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
        .fold(q!(|| 0), q!(|count, string: String| *count += string.len()))
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
    graph: &'a HfBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = simple_cluster(graph, &cli, &cli);
    graph.build(node_id)
}

#[stageleft::entry]
pub fn map_reduce_runtime<'a>(
    graph: &'a HfBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    node_id: RuntimeData<usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = map_reduce(graph, &cli, &cli);
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
        let (node, cluster) = super::simple_cluster(
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

        let node_stdout = node.stdout().await;
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
                    format!("cluster received: ({}, {})", i, j)
                );
            }
        }

        let node_outs_res = futures::future::join_all((0..10).map(|_| node_stdout.recv())).await;
        let mut node_outs: Vec<String> = node_outs_res.into_iter().map(|o| o.unwrap()).collect();
        node_outs.sort();

        for (i, n) in node_outs.into_iter().enumerate() {
            assert_eq!(
                n,
                format!("node received: ({}, ({}, {}))", i / 5, i / 5, i % 5)
            );
        }
    }
}
