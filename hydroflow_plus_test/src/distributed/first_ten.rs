use hydroflow_plus::*;
use location::external_process::ExternalBincodeSink;
use location::ExternalProcess;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SendOverNetwork {
    pub n: u32,
}

pub struct P1 {}
pub struct P2 {}

pub fn first_ten_distributed<'a>(
    external: &ExternalProcess<'a, ()>,
    process: &Process<'a, P1>,
    second_process: &Process<'a, P2>,
) -> ExternalBincodeSink<String> {
    let (numbers_external_port, numbers_external) = external.source_external_bincode(process);
    numbers_external.for_each(q!(|n| println!("hi: {:?}", n)));

    let numbers = process.source_iter(q!(0..10));
    numbers
        .map(q!(|n| SendOverNetwork { n }))
        .send_bincode(second_process)
        .for_each(q!(|n| println!("{}", n.n)));

    numbers_external_port
}

#[cfg(test)]
mod tests {
    use futures::SinkExt;
    use hydro_deploy::Deployment;
    use hydroflow_plus::deploy::DeployCrateWrapper;

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();

        let builder = hydroflow_plus::FlowBuilder::new();
        let external = builder.external_process();
        let p1 = builder.process();
        let p2 = builder.process();
        let external_port = super::first_ten_distributed(&external, &p1, &p2);

        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        let nodes = built
            .with_process(&p1, deployment.Localhost())
            .with_process(&p2, deployment.Localhost())
            .with_external(&external, deployment.Localhost())
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut external_port = nodes.connect_sink_bincode(external_port).await;

        let mut first_node_stdout = nodes.get_process(&p1).stdout().await;
        let mut second_node_stdout = nodes.get_process(&p2).stdout().await;

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
