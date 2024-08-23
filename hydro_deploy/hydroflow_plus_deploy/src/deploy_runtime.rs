use hydroflow_plus::util::deploy::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged, DeployPorts,
};
use stageleft::{q, Quoted, RuntimeData};

use crate::HydroflowPlusMeta;

pub fn cluster_members<'a>(of_cluster: usize) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
    let cli: RuntimeData<&DeployPorts<HydroflowPlusMeta>> =
        RuntimeData::new("__hydroflow_plus_trybuild_cli");
    q!(cli.meta.clusters.get(&of_cluster).unwrap())
}

pub fn cluster_self_id<'a>() -> impl Quoted<'a, u32> + Copy + 'a {
    let cli: RuntimeData<&DeployPorts<HydroflowPlusMeta>> =
        RuntimeData::new("__hydroflow_plus_trybuild_cli");
    q!(cli
        .meta
        .cluster_id
        .expect("Tried to read Cluster ID on a non-cluster node"))
}

pub fn deploy_o2o(p1_port: &str, p2_port: &str) -> (syn::Expr, syn::Expr) {
    let env: RuntimeData<&DeployPorts<HydroflowPlusMeta>> =
        RuntimeData::new("__hydroflow_plus_trybuild_cli");
    (
        {
            q!({
                env.port(p1_port)
                    .connect_local_blocking::<ConnectedDirect>()
                    .into_sink()
            })
            .splice()
        },
        {
            q!({
                env.port(p2_port)
                    .connect_local_blocking::<ConnectedDirect>()
                    .into_source()
            })
            .splice()
        },
    )
}

pub fn deploy_o2m(p1_port: &str, c2_port: &str) -> (syn::Expr, syn::Expr) {
    let env: RuntimeData<&DeployPorts<HydroflowPlusMeta>> =
        RuntimeData::new("__hydroflow_plus_trybuild_cli");
    (
        {
            q!({
                env.port(p1_port)
                    .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                    .into_sink()
            })
            .splice()
        },
        {
            q!({
                env.port(c2_port)
                    .connect_local_blocking::<ConnectedDirect>()
                    .into_source()
            })
            .splice()
        },
    )
}

pub fn deploy_m2o(c1_port: &str, p2_port: &str) -> (syn::Expr, syn::Expr) {
    let env: RuntimeData<&DeployPorts<HydroflowPlusMeta>> =
        RuntimeData::new("__hydroflow_plus_trybuild_cli");
    (
        {
            q!({
                env.port(c1_port)
                    .connect_local_blocking::<ConnectedDirect>()
                    .into_sink()
            })
            .splice()
        },
        {
            q!({
                env.port(p2_port)
                    .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                    .into_source()
            })
            .splice()
        },
    )
}

pub fn deploy_m2m(c1_port: &str, c2_port: &str) -> (syn::Expr, syn::Expr) {
    let env: RuntimeData<&DeployPorts<HydroflowPlusMeta>> =
        RuntimeData::new("__hydroflow_plus_trybuild_cli");
    (
        {
            q!({
                env.port(c1_port)
                    .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                    .into_sink()
            })
            .splice()
        },
        {
            q!({
                env.port(c2_port)
                    .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                    .into_source()
            })
            .splice()
        },
    )
}
