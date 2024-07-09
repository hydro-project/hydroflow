use hydroflow_plus::*;
use serde::{Deserialize, Serialize};
use stageleft::*;

#[derive(Serialize, Deserialize)]
struct SendOverNetwork {
    pub n: u32,
}

pub fn first_ten_distributed<'a, D: Deploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    process_spec: &impl ProcessSpec<'a, D>,
) -> D::Process {
    let process = flow.process(process_spec);
    let second_process = flow.process(process_spec);

    let numbers = flow.source_iter(&process, q!(0..10));
    numbers
        .map(q!(|n| SendOverNetwork { n }))
        .send_bincode(&second_process)
        .for_each(q!(|n: SendOverNetwork| println!("{}", n.n))); // TODO(shadaj): why is the explicit type required here?

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

        // if we drop this, we drop the references to the deployment nodes
        let built = builder.extract();

        insta::assert_debug_snapshot!(built.ir());

        deployment.deploy().await.unwrap();

        let second_node_stdout = second_node.stdout().await;

        deployment.start().await.unwrap();

        for i in 0..10 {
            assert_eq!(second_node_stdout.recv().await.unwrap(), i.to_string());
        }
    }
}
