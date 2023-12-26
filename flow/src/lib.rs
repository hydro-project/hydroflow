stageleft::stageleft_crate!(flow_macro);

use hydroflow_plus::node::{Deploy, HfNode, NodeBuilder, ClusterBuilder, HfCluster};
use hydroflow_plus::GraphBuilder;
use stageleft::{q, Quoted, RuntimeData};

use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

pub fn partitioned_char_counter<'a, D: Deploy<'a>>(
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
        .fold(q!(|| 0), q!(|count, string: String| *count += string.len()))
        .inspect(q!(|count| println!("partition count: {}", count)))
        .send_bincode_tagged(&node)
        .persist()
        .map(q!(|(_mid, count)| count))
        .fold(q!(|| 0), q!(|total, count| *total += count))
        .for_each(q!(|data| println!("total: {}", data)));

    (node, cluster)
}

#[stageleft::entry]
pub fn partitioned_char_counter_runtime<'a>(
    graph: &'a GraphBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = partitioned_char_counter(graph, &cli, &cli);
    graph.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus::futures::StreamExt;
    use hydroflow_plus_cli_integration::{CLIDeployNodeBuilder, DeployCrateWrapper, CLIDeployClusterBuilder};

    #[tokio::test]
    async fn partitioned_char_counter() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let deployment_cell = RefCell::new(deployment);
        let builder = hydroflow_plus::GraphBuilder::new();
        let (leader, _) = super::partitioned_char_counter(
            &builder,
            &CLIDeployNodeBuilder::new(|| {
                deployment_cell.borrow_mut().add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("my_dataflow")
                        .profile("dev"),
                )
            }),
            &CLIDeployClusterBuilder::new(|| {
                (0..2).map(|_| {
                    deployment_cell.borrow_mut().add_service(
                        HydroflowCrate::new(".", localhost.clone())
                            .bin("my_dataflow")
                            .profile("dev"),
                    )
                }).collect()
            }),
        );

        let mut deployment = deployment_cell.into_inner();
        deployment.deploy().await.unwrap();

        let mut leader_stdout = leader.stdout().await;

        deployment.start().await.unwrap();

        while let Some(line) = leader_stdout.next().await {
            if line == "total: 9" {
                return;
            }
        }

        panic!("did not find total: 9");
    }
}
