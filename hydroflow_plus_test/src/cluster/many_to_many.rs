use hydroflow_plus::*;
use stageleft::*;

pub fn many_to_many<'a, D: Deploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    cluster_spec: &impl ClusterSpec<'a, D>,
) -> D::Cluster
where
    D::ClusterId: std::fmt::Debug,
{
    let cluster = flow.cluster(cluster_spec);
    flow.source_iter(&cluster, q!(0..2))
        .broadcast_bincode(&cluster)
        .for_each(q!(|n| println!("cluster received: {:?}", n)));

    cluster
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn many_to_many_runtime<'a>(
    flow: FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = many_to_many(&flow, &cli);
    flow.with_default_optimize()
        .compile()
        .with_dynamic_id(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus_cli_integration::{DeployClusterSpec, DeployCrateWrapper};

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

        insta::assert_debug_snapshot!(builder.finalize().ir());

        let mut deployment = deployment.into_inner();

        deployment.deploy().await.unwrap();

        let cluster_stdouts =
            futures::future::join_all(cluster.members.iter().map(|node| node.stdout())).await;

        deployment.start().await.unwrap();

        for mut node_stdout in cluster_stdouts {
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
