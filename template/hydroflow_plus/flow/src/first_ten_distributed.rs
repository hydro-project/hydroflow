use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten_distributed<'a, D: Deploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) -> D::Process {
    let process = flow.process(process_spec);
    let second_process = flow.process(process_spec);

    let numbers = flow.source_iter(&process, q!(0..10));
    numbers
        .send_bincode(&second_process)
        .for_each(q!(|n| println!("{}", n)));

    second_process
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn first_ten_distributed_runtime<'a>(
    flow: FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = first_ten_distributed(&flow, &cli);
    flow.extract()
        .optimize_default()
        .with_dynamic_id(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus::futures::StreamExt;
    use hydroflow_plus_cli_integration::{DeployCrateWrapper, DeployProcessSpec};
    use tokio_stream::wrappers::UnboundedReceiverStream;

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let flow = hydroflow_plus::FlowBuilder::new();
        let second_process = super::first_ten_distributed(
            &flow,
            &DeployProcessSpec::new(|| {
                deployment.add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("first_ten_distributed")
                        .profile("dev"),
                )
            }),
        );

        deployment.deploy().await.unwrap();

        let mut second_process_stdout = second_process.stdout().await;

        deployment.start().await.unwrap();

        assert_eq!(
            UnboundedReceiverStream::new(second_process_stdout).take(10).collect::<Vec<_>>().await,
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
    }
}
