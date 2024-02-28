use hydroflow_plus::*;
use stageleft::*;

pub fn first_ten<'a, D: LocalDeploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) {
    let process = flow.process(process_spec);
    let numbers = process.source_iter(q!(0..10));
    numbers.for_each(q!(|n| println!("{}", n)));
}

#[stageleft::entry]
pub fn first_ten_runtime<'a>(
    flow: &'a FlowBuilder<'a, SingleProcessGraph>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    first_ten(flow, &());
    flow.build_single()
}

pub fn first_ten_distributed<'a, D: Deploy<'a>>(
    flow: &'a FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) -> D::Process {
    let process = flow.process(process_spec);
    let second_process = flow.process(process_spec);

    let numbers = process.source_iter(q!(0..10));
    numbers
        .send_bincode(&second_process)
        .for_each(q!(|n| println!("{}", n)));

    second_process
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn first_ten_distributed_runtime<'a>(
    flow: &'a FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = first_ten_distributed(flow, &cli);
    flow.build(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydro_deploy::{Deployment, HydroflowCrate};
    use hydroflow_plus_cli_integration::{DeployCrateWrapper, DeployProcessSpec};

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();
        let localhost = deployment.Localhost();

        let builder = hydroflow_plus::FlowBuilder::new();
        let second_node = super::first_ten_distributed(
            &builder,
            &DeployProcessSpec::new(|| {
                deployment.add_service(
                    HydroflowCrate::new(".", localhost.clone())
                        .bin("first_ten_distributed")
                        .profile("dev"),
                )
            }),
        );

        insta::assert_debug_snapshot!(builder.ir());

        deployment.deploy().await.unwrap();

        let second_node_stdout = second_node.stdout().await;

        deployment.start().await.unwrap();

        for i in 0..10 {
            assert_eq!(second_node_stdout.recv().await.unwrap(), i.to_string());
        }
    }
}
