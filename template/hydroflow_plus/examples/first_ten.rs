use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();

    let flow = hydroflow_plus::FlowBuilder::new();
    let process = flow.process();
    hydroflow_plus_template::first_ten::first_ten(&process);

    let _nodes = flow
        .with_default_optimize()
        .with_process(&process, deployment.Localhost())
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
