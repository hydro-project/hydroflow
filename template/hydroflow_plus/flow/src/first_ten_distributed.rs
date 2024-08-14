use hydroflow_plus::*;
use stageleft::*;

pub struct P1 {}
pub struct P2 {}

pub fn first_ten_distributed(flow: &FlowBuilder) -> (Process<P1>, Process<P2>) {
    let process = flow.process::<P1>();
    let second_process = flow.process::<P2>();

    let numbers = flow.source_iter(&process, q!(0..10));
    numbers
        .send_bincode(&second_process)
        .for_each(q!(|n| println!("{}", n)));

    (process, second_process)
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn first_ten_distributed_runtime<'a>(
    flow: FlowBuilder<'a>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = first_ten_distributed(&flow);
    flow.with_default_optimize()
        .compile::<CLIRuntime>(&cli)
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
        let (p1, p2) = super::first_ten_distributed(&flow);

        let nodes = flow.with_default_optimize()
            .with_process(p1, DeployProcessSpec::new({
                HydroflowCrate::new(".", localhost.clone())
                    .bin("first_ten_distributed")
                    .profile("dev")
            }))
            .with_process(p2, DeployProcessSpec::new({
                HydroflowCrate::new(".", localhost.clone())
                    .bin("first_ten_distributed")
                    .profile("dev")
            }))
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let second_process_stdout = second_process.stdout().await;

        deployment.start().await.unwrap();

        assert_eq!(
            UnboundedReceiverStream::new(second_process_stdout)
                .take(10)
                .collect::<Vec<_>>()
                .await,
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
    }
}
