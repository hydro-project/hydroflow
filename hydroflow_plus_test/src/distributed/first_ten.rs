use hydroflow_plus::*;
use serde::{Deserialize, Serialize};
use stageleft::*;

#[derive(Serialize, Deserialize)]
struct SendOverNetwork {
    pub n: u32,
}

pub struct P1 {}
pub struct P2 {}

pub fn first_ten_distributed(flow: &FlowBuilder) -> (Process<P1>, Process<P2>) {
    let process = flow.process::<P1>();
    let second_process = flow.process::<P2>();

    let numbers = flow.source_iter(&process, q!(0..10));
    numbers
        .map(q!(|n| SendOverNetwork { n }))
        .send_bincode(&second_process)
        .for_each(q!(|n: SendOverNetwork| println!("{}", n.n))); // TODO(shadaj): why is the explicit type required here?

    (process, second_process)
}

#[cfg(test)]
mod tests {
    use hydro_deploy::Deployment;
    use hydroflow_plus::deploy::{DeployCrateWrapper, TrybuildHost};

    #[tokio::test]
    async fn first_ten_distributed() {
        let mut deployment = Deployment::new();

        let builder = hydroflow_plus::FlowBuilder::new();
        let (first_node, second_node) = super::first_ten_distributed(&builder);

        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        // if we drop this, we drop the references to the deployment nodes
        let nodes = built
            .with_process(&first_node, TrybuildHost::new(deployment.Localhost()))
            .with_process(&second_node, TrybuildHost::new(deployment.Localhost()))
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut second_node_stdout = nodes.get_process(&second_node).stdout().await;

        deployment.start().await.unwrap();

        for i in 0..10 {
            assert_eq!(second_node_stdout.recv().await.unwrap(), i.to_string());
        }
    }
}

#[cfg(test)]
mod other_tests {
    use hydro_deploy::Deployment;
    use hydroflow_plus::deploy::{DeployCrateWrapper, HydroDeploy, TrybuildHost};
    use hydroflow_plus::FlowBuilder;
    use serde::{Deserialize, Serialize};
    use stageleft::q;

    #[derive(Serialize, Deserialize)]
    struct SendOverNetwork {
        pub n: u32,
    }

    struct P1 {}
    struct P2 {}

    #[tokio::test]
    async fn first_ten_distributed_other() {
        hydroflow_plus::deploy::init_test();
        let mut deployment = Deployment::new();

        let flow = FlowBuilder::new();
        let first_node = flow.process::<P1>();
        let second_node = flow.process::<P2>();

        let numbers = flow.source_iter(&first_node, q!(0..10));
        numbers
            .map(q!(|n| SendOverNetwork { n }))
            .send_bincode(&second_node)
            .for_each(q!(|n: SendOverNetwork| println!("{}", n.n))); // TODO(shadaj): why is the explicit type required here?

        let built = flow.with_default_optimize();

        // if we drop this, we drop the references to the deployment nodes
        let nodes = built
            .with_process::<_, HydroDeploy>(&first_node, TrybuildHost::new(deployment.Localhost()))
            .with_process(&second_node, TrybuildHost::new(deployment.Localhost()))
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut second_node_stdout = nodes.get_process(&second_node).stdout().await;

        deployment.start().await.unwrap();

        for i in 0..10 {
            assert_eq!(second_node_stdout.recv().await.unwrap(), i.to_string());
        }
    }
}
