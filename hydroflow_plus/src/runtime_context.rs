use hydroflow::scheduled::context::Context;
use proc_macro2::TokenStream;
use quote::quote;
use stageleft::runtime_support::FreeVariableWithContext;

use crate::{Cluster, Process};

pub static RUNTIME_CONTEXT: RuntimeContext = RuntimeContext { _private: () };

#[derive(Clone, Copy)]
pub struct RuntimeContext {
    _private: (),
}

impl<'a, P> FreeVariableWithContext<Process<'a, P>> for RuntimeContext {
    type O = &'a Context;

    fn to_tokens(self, _ctx: &Process<'a, P>) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(&context)))
    }
}

impl<'a, P> FreeVariableWithContext<Cluster<'a, P>> for RuntimeContext {
    type O = &'a Context;

    fn to_tokens(self, _ctx: &Cluster<'a, P>) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(&context)))
    }
}

#[cfg(test)]
mod tests {
    use hydro_deploy::Deployment;
    use hydroflow::futures::StreamExt;

    use crate::*;

    struct P1 {}

    #[tokio::test]
    async fn runtime_context() {
        let mut deployment = Deployment::new();

        let flow = FlowBuilder::new();
        let node = flow.process::<P1>();
        let external = flow.external_process::<()>();

        let out_port = node
            .source_iter(q!(0..5))
            .map(q!(|v| (v, RUNTIME_CONTEXT.current_tick().0)))
            .send_bincode_external(&external);

        let nodes = flow
            .with_process(&node, deployment.Localhost())
            .with_external(&external, deployment.Localhost())
            .deploy(&mut deployment);

        deployment.deploy().await.unwrap();

        let mut external_out = nodes.connect_source_bincode(out_port).await;

        deployment.start().await.unwrap();

        for i in 0..5 {
            assert_eq!(external_out.next().await.unwrap(), (i, 0));
        }
    }
}
