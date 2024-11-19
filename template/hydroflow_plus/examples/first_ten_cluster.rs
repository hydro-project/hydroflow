use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();

    let flow = hydroflow_plus::FlowBuilder::new();
    let leader = flow.process();
    let workers = flow.cluster();
    hydroflow_plus_template::first_ten_cluster::first_ten_cluster(&leader, &workers);

    let _nodes = flow
        .with_process(&leader, deployment.Localhost())
        // highlight-next-line
        .with_cluster(&workers, vec![deployment.Localhost(); 4])
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
