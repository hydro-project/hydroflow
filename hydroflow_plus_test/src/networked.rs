use hydroflow_plus::bytes::BytesMut;
use hydroflow_plus::scheduled::graph::Hydroflow;
use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus::{FlowBuilder, *};
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};
use stageleft::{q, Quoted, RuntimeData};

pub fn networked_basic<'a, D: Deploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) -> (D::ProcessPort, D::Process, D::Process) {
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

    (source_zero_port, process_zero, process_one)
}

#[stageleft::entry]
pub fn networked_basic_runtime<'a>(
    flow: &'a FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = networked_basic(flow, &cli);
    flow.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus::futures::SinkExt;
    use hydroflow_plus::util::cli::ConnectedSink;
    use hydroflow_plus_cli_integration::{DeployCrateWrapper, DeployProcessSpec};

    #[tokio::test]
    async fn networked_basic() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (source_zero_port, _, node_one) = super::networked_basic(
            &builder,
            &DeployProcessSpec::new(|| {
                deployment.add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("networked_basic")
                        .profile("dev"),
                )
            }),
        );

        let port_to_zero = source_zero_port
            .create_sender(&mut deployment, &localhost)
            .await;

        deployment.deploy().await.unwrap();

        let mut conn_to_zero = port_to_zero.connect().await.into_sink();
        let node_one_stdout = node_one.stdout().await;

        deployment.start().await.unwrap();

        conn_to_zero.send("hello world!".into()).await.unwrap();

        assert_eq!(
            node_one_stdout.recv().await.unwrap(),
            "node one received: \"hello world!\""
        );
    }
}
