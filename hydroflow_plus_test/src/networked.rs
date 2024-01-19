use hydroflow_plus::bytes::BytesMut;
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus::*;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};
use stageleft::{q, Quoted, RuntimeData};

pub struct NetworkedBasicIO<'a, D: Deploy<'a>> {
    pub source_zero_port: D::ProcessPort,
    pub process_zero: D::Process,
    pub process_one: D::Process,
    pub cluster_port: D::ClusterPort,
    pub cluster: D::Cluster,
}

pub fn networked_basic<'a, D: Deploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
    cluster_spec: &impl ClusterSpec<'a, D>,
) -> NetworkedBasicIO<'a, D> {
    let process_zero = flow.process(process_spec);
    let process_one = flow.process(process_spec);

    let (source_zero_port, source_zero) = process_zero.source_external();

    source_zero
        .map(q!(|v| v.unwrap().freeze()))
        .send_bytes(&process_one)
        .for_each(q!(|v: Result<BytesMut, _>| {
            println!(
                "node one received: {:?}",
                std::str::from_utf8(&v.unwrap()).unwrap()
            );
        }));

    let cluster = flow.cluster(cluster_spec);
    let (cluster_port, cluster_stream) = cluster.many_source_external::<D::Process>();
    cluster_stream.for_each(q!(|v: Result<BytesMut, _>| {
        println!(
            "cluster received: {:?}",
            std::str::from_utf8(&v.unwrap()).unwrap()
        );
    }));

    NetworkedBasicIO {
        source_zero_port,
        process_zero,
        process_one,
        cluster_port,
        cluster,
    }
}

#[stageleft::entry]
pub fn networked_basic_runtime<'a>(
    flow: &'a FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = networked_basic(flow, &cli, &cli);
    flow.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus::futures::SinkExt;
    use hydroflow_plus::util::cli::ConnectedSink;
    use hydroflow_plus_cli_integration::{
        DeployClusterSpec, DeployCrateWrapper, DeployProcessSpec,
    };

    #[tokio::test]
    async fn networked_basic() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::FlowBuilder::new();
        let deployment = RefCell::new(deployment);
        let io = super::networked_basic(
            &builder,
            &DeployProcessSpec::new(|| {
                deployment.borrow_mut().add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("networked_basic")
                        .profile("dev"),
                )
            }),
            &DeployClusterSpec::new(|| {
                vec![deployment.borrow_mut().add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("networked_basic")
                        .profile("dev"),
                )]
            }),
        );

        let mut deployment = deployment.into_inner();

        let port_to_zero = io
            .source_zero_port
            .create_sender(&mut deployment, &localhost)
            .await;

        let ports_to_cluster = io
            .cluster_port
            .create_senders(&mut deployment, &localhost)
            .await;

        deployment.deploy().await.unwrap();

        let mut conn_to_zero = port_to_zero.connect().await.into_sink();
        let node_one_stdout = io.process_one.stdout().await;

        let mut conn_to_cluster = ports_to_cluster[0].connect().await.into_sink();
        let cluster_stdout = io.cluster.members[0].stdout().await;

        deployment.start().await.unwrap();

        conn_to_zero.send("hello world!".into()).await.unwrap();
        conn_to_cluster.send("hello cluster!".into()).await.unwrap();

        assert_eq!(
            node_one_stdout.recv().await.unwrap(),
            "node one received: \"hello world!\""
        );

        assert_eq!(
            cluster_stdout.recv().await.unwrap(),
            "cluster received: \"hello cluster!\""
        );
    }
}
