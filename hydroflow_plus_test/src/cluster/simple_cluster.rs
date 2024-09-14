use hydroflow_plus::*;
use stageleft::*;

pub fn decouple_cluster(flow: &FlowBuilder) -> (Cluster<()>, Cluster<()>) {
    let cluster1 = flow.cluster();
    let cluster2 = flow.cluster();
    let cluster_self_id = flow.cluster_self_id(&cluster2);
    flow.source_iter(&cluster1, q!(0..2))
        .decouple_cluster(&cluster2)
        .for_each(q!(move |(id, message)| println!(
            "I received from {}, my self id is {}, my message is {}",
            id, cluster_self_id, message
        )));
    (cluster1, cluster2)
}

pub fn decouple_process(flow: &FlowBuilder) -> (Process<()>, Process<()>) {
    let process1 = flow.process();
    let process2 = flow.process();
    flow.source_iter(&process1, q!(0..3))
        .decouple_process(&process2)
        .for_each(q!(|message| println!("I received message is {}", message)));
    (process1, process2)
}

pub fn simple_cluster(flow: &FlowBuilder) -> (Process<()>, Cluster<()>) {
    let process = flow.process();
    let cluster = flow.cluster();

    let numbers = process.source_iter(q!(0..5));
    let ids = process.source_iter(cluster.members()).map(q!(|&id| id));

    let cluster_self_id = cluster.self_id();

    ids.cross_product(numbers)
        .map(q!(|(id, n)| (id, (id, n))))
        .send_bincode(&cluster)
        .tick_batch()
        .inspect(q!(move |n| println!(
            "cluster received: {:?} (self cluster id: {})",
            n, cluster_self_id
        )))
        .all_ticks()
        .send_bincode(&process)
        .for_each(q!(|(id, d)| println!("node received: ({}, {:?})", id, d)));

    (process, cluster)
}

#[cfg(test)]
mod tests {
    use hydro_deploy::Deployment;
    use hydroflow_plus::deploy::{DeployCrateWrapper, TrybuildHost};

    #[tokio::test]
    async fn decouple_cluster() {
        let mut deployment = Deployment::new();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (cluster1, cluster2) = super::decouple_cluster(&builder);
        let built = builder.with_default_optimize();

        let nodes = built
            .with_cluster(
                &cluster1,
                (0..3)
                    .map(|_| TrybuildHost::new(deployment.Localhost()))
                    .collect::<Vec<_>>(),
            )
            .with_cluster(
                &cluster2,
                (0..3)
                    .map(|_| TrybuildHost::new(deployment.Localhost()))
                    .collect::<Vec<_>>(),
            )
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        deployment.start().await.unwrap();
    }

    #[tokio::test]
    async fn decouple_process() {
        let mut deployment = Deployment::new();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (process1, process2) = super::decouple_process(&builder);
        let built = builder.with_default_optimize();

        let nodes = built
            .with_process(&process1, TrybuildHost::new(deployment.Localhost()))
            .with_process(&process2, TrybuildHost::new(deployment.Localhost()))
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        deployment.start().await.unwrap();
    }
    #[tokio::test]
    async fn simple_cluster() {
        let mut deployment = Deployment::new();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (node, cluster) = super::simple_cluster(&builder);
        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        let nodes = built
            .with_process(&node, TrybuildHost::new(deployment.Localhost()))
            .with_cluster(
                &cluster,
                (0..2)
                    .map(|_| TrybuildHost::new(deployment.Localhost()))
                    .collect::<Vec<_>>(),
            )
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut node_stdout = nodes.get_process(&node).stdout().await;
        let cluster_stdouts = futures::future::join_all(
            nodes
                .get_cluster(&cluster)
                .members()
                .iter()
                .map(|node| node.stdout()),
        )
        .await;

        deployment.start().await.unwrap();

        for (i, mut stdout) in cluster_stdouts.into_iter().enumerate() {
            for j in 0..5 {
                assert_eq!(
                    stdout.recv().await.unwrap(),
                    format!("cluster received: ({}, {}) (self cluster id: {})", i, j, i)
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
}
