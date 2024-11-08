use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();

    let flow = hydroflow_plus::FlowBuilder::new();
    let (p1, p2) = hydroflow_plus_template::first_ten_distributed::first_ten_distributed(&flow);

    let _nodes = flow
        .with_process(&p1, deployment.Localhost())
        .with_process(&p2, deployment.Localhost())
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
