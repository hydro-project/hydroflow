use hydroflow_plus::*;
use location::{ExternalBincodePort, ExternalProcess};
use serde::{Deserialize, Serialize};
use stageleft::*;

#[derive(Serialize, Deserialize)]
struct SendOverNetwork {
    pub n: u32,
}

pub struct P1 {}
pub struct P2 {}

pub fn first_ten_distributed(
    flow: &FlowBuilder,
) -> (
    ExternalProcess<()>,
    ExternalBincodePort<String>,
    Process<P1>,
    Process<P2>,
) {
    let external_process = flow.external_process::<()>();
    let process = flow.process::<P1>();
    let second_process = flow.process::<P2>();

    let (numbers_external_port, numbers_external) =
        flow.source_external_bincode(&external_process, &process);
    numbers_external.for_each(q!(|n| println!("hi: {:?}", n)));

    let numbers = flow.source_iter(&process, q!(0..10));
    numbers
        .map(q!(|n| SendOverNetwork { n }))
        .send_bincode(&second_process)
        .for_each(q!(|n: SendOverNetwork| println!("{}", n.n))); // TODO(shadaj): why is the explicit type required here?

    (
        external_process,
        numbers_external_port,
        process,
        second_process,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::SinkExt;
    use hydro_deploy::{Deployment, Host};
    use hydroflow_plus_deploy::{DeployCrateWrapper, TrybuildHost};

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (external_process, external_port, first_node, second_node) =
            super::first_ten_distributed(&builder);

        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        // if we drop this, we drop the references to the deployment nodes
        let nodes = built
            .with_process(&first_node, TrybuildHost::new(deployment.Localhost()))
            .with_process(&second_node, TrybuildHost::new(deployment.Localhost()))
            .with_external(&external_process, deployment.Localhost() as Arc<dyn Host>)
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut external_port = nodes.connect_sink_bincode(external_port).await;

        let mut first_node_stdout = nodes.get_process(&first_node).stdout().await;
        let mut second_node_stdout = nodes.get_process(&second_node).stdout().await;

        deployment.start().await.unwrap();

        external_port
            .send("this is some string".to_string())
            .await
            .unwrap();
        assert_eq!(
            first_node_stdout.recv().await.unwrap(),
            "hi: \"this is some string\""
        );

        for i in 0..10 {
            assert_eq!(second_node_stdout.recv().await.unwrap(), i.to_string());
        }
    }
}
