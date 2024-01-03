use hydroflow_plus::node::*;
use hydroflow_plus::*;
use stageleft::*;

pub fn simple_cluster<'a, D: Deploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
    cluster_spec: &impl ClusterSpec<'a, D>,
) -> (D::Process, D::Cluster) {
    let process = flow.process(process_spec);
    let cluster = flow.cluster(cluster_spec);

    let numbers = process.source_iter(q!(0..5));
    let ids = process.source_iter(cluster.ids()).map(q!(|&id| id));

    ids.cross_product(&numbers)
        .map(q!(|(id, n)| (id, (id, n))))
        .demux_bincode(&cluster)
        .inspect(q!(|n| println!("cluster received: {:?}", n)))
        .send_bincode_tagged(&process)
        .for_each(q!(|(id, d)| println!("node received: ({}, {:?})", id, d)));

    (process, cluster)
}

pub fn many_to_many<'a, D: Deploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    cluster_spec: &impl ClusterSpec<'a, D>,
) -> D::Cluster {
    let cluster = flow.cluster(cluster_spec);
    cluster
        .source_iter(q!(0..2))
        .broadcast_bincode_tagged(&cluster)
        .for_each(q!(|n| println!("cluster received: {:?}", n)));

    cluster
}

pub fn map_reduce<'a, D: Deploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
    cluster_spec: &impl ClusterSpec<'a, D>,
) -> (D::Process, D::Cluster) {
    let process = flow.process(process_spec);
    let cluster = flow.cluster(cluster_spec);

    let words = process
        .source_iter(q!(vec!["abc", "abc", "xyz"]))
        .map(q!(|s| s.to_string()));

    let all_ids_vec = cluster.ids();
    let words_partitioned = words
        .enumerate()
        .map(q!(|(i, w)| ((i % all_ids_vec.len()) as u32, w)));

    words_partitioned
        .demux_bincode(&cluster)
        .tick_batch()
        .fold(q!(|| 0), q!(|count, string| *count += string.len()))
        .inspect(q!(|count| println!("partition count: {}", count)))
        .send_bincode_tagged(&process)
        .all_ticks()
        .map(q!(|(_mid, count)| count))
        .fold(q!(|| 0), q!(|total, count| *total += count))
        .for_each(q!(|data| println!("total: {}", data)));

    (process, cluster)
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn simple_cluster_runtime<'a>(
    flow: &'a FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = simple_cluster(flow, &cli, &cli);
    flow.build(q!(cli.meta.subgraph_id))
}

#[stageleft::entry]
pub fn many_to_many_runtime<'a>(
    flow: &'a FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = many_to_many(flow, &cli);
    flow.build(q!(cli.meta.subgraph_id))
}

#[stageleft::entry]
pub fn map_reduce_runtime<'a>(
    flow: &'a FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = map_reduce(flow, &cli, &cli);
    flow.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus_cli_integration::{
        DeployClusterSpec, DeployCrateWrapper, DeployProcessSpec,
    };

    #[tokio::test]
    async fn simple_cluster() {
        let deployment = RefCell::new(Deployment::new());
        let localhost = deployment.borrow_mut().Localhost();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (node, cluster) = super::simple_cluster(
            &builder,
            &DeployProcessSpec::new(|| {
                deployment.borrow_mut().add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("simple_cluster")
                        .profile("dev"),
                )
            }),
            &DeployClusterSpec::new(|| {
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
            futures::future::join_all(cluster.members.iter().map(|node| node.stdout())).await;

        deployment.start().await.unwrap();

        for (i, stdout) in cluster_stdouts.into_iter().enumerate() {
            for j in 0..5 {
                assert_eq!(
                    stdout.recv().await.unwrap(),
                    format!("cluster received: ({}, {})", i, j)
                );
            }
        }

        let mut node_outs = vec![];
        for _i in 0..10 {
            node_outs.push(node_stdout.recv().await.unwrap());
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

        let builder = hydroflow_plus::FlowBuilder::new();
        let cluster = super::many_to_many(
            &builder,
            &DeployClusterSpec::new(|| {
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
            futures::future::join_all(cluster.members.iter().map(|node| node.stdout())).await;

        deployment.start().await.unwrap();

        for node_stdout in cluster_stdouts {
            let mut node_outs = vec![];
            for _i in 0..4 {
                node_outs.push(node_stdout.recv().await.unwrap());
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
