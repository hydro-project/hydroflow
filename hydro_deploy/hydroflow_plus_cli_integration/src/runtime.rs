use std::cell::RefCell;
use std::rc::Rc;

use hydroflow_plus::deploy::{ClusterSpec, Deploy, Node, ProcessSpec};
use hydroflow_plus::lang::graph::HydroflowGraph;
use hydroflow_plus::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged, HydroCLI,
};
use stageleft::{q, Quoted, RuntimeData};

use super::HydroflowPlusMeta;

pub struct CLIRuntime {}

impl<'a> Deploy<'a> for CLIRuntime {
    type InstantiateEnv = ();
    type CompileEnv = RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>;
    type Process = CLIRuntimeNode;
    type Cluster = CLIRuntimeCluster;
    type Meta = ();
    type GraphId = usize;
    type ProcessPort = String;
    type ClusterPort = String;

    fn has_default_node() -> bool {
        true
    }

    fn default_process(_id: usize) -> Self::Process {
        CLIRuntimeNode {
            next_port: Rc::new(RefCell::new(0)),
        }
    }

    fn default_cluster(_id: usize) -> Self::Cluster {
        CLIRuntimeCluster {
            next_port: Rc::new(RefCell::new(0)),
        }
    }

    fn allocate_process_port(process: &Self::Process) -> Self::ProcessPort {
        process.next_port()
    }

    fn allocate_cluster_port(cluster: &Self::Cluster) -> Self::ClusterPort {
        cluster.next_port()
    }

    fn o2o_sink_source(
        env: &Self::CompileEnv,
        _p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        _p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> (syn::Expr, syn::Expr) {
        let env = *env;
        (
            {
                let port = p1_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = p2_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn o2o_connect(
        _p1: &Self::Process,
        _p1_port: &Self::ProcessPort,
        _p2: &Self::Process,
        _p2_port: &Self::ProcessPort,
    ) {
        panic!()
    }

    fn o2m_sink_source(
        env: &Self::CompileEnv,
        _p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        _c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) -> (syn::Expr, syn::Expr) {
        let env = *env;
        (
            {
                let port = p1_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = c2_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn o2m_connect(
        _p1: &Self::Process,
        _p1_port: &Self::ProcessPort,
        _c2: &Self::Cluster,
        _c2_port: &Self::ClusterPort,
    ) {
        panic!()
    }

    fn m2o_sink_source(
        env: &Self::CompileEnv,
        _c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        _p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> (syn::Expr, syn::Expr) {
        let env = *env;
        (
            {
                let port = c1_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = p2_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn m2o_connect(
        _c1: &Self::Cluster,
        _c1_port: &Self::ClusterPort,
        _p2: &Self::Process,
        _p2_port: &Self::ProcessPort,
    ) {
        panic!()
    }

    fn m2m_sink_source(
        env: &Self::CompileEnv,
        _c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        _c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) -> (syn::Expr, syn::Expr) {
        let env = *env;
        (
            {
                let port = c1_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = c2_port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn m2m_connect(
        _c1: &Self::Cluster,
        _c1_port: &Self::ClusterPort,
        _c2: &Self::Cluster,
        _c2_port: &Self::ClusterPort,
    ) {
        panic!()
    }

    fn cluster_ids(
        env: &Self::CompileEnv,
        of_cluster: usize,
    ) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        let cli = *env;
        q!(cli.meta.clusters.get(&of_cluster).unwrap())
    }

    fn cluster_self_id(env: &Self::CompileEnv) -> impl Quoted<'a, u32> + Copy + 'a {
        let cli = *env;
        q!(cli
            .meta
            .cluster_id
            .expect("Tried to read Cluster ID on a non-cluster node"))
    }
}

#[derive(Clone)]
pub struct CLIRuntimeNode {
    next_port: Rc<RefCell<usize>>,
}

impl Node for CLIRuntimeNode {
    type Port = String;
    type Meta = ();
    type InstantiateEnv = ();

    fn next_port(&self) -> String {
        let next_send_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_send_port)
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}

    fn instantiate(
        &self,
        _env: &mut Self::InstantiateEnv,
        _meta: &mut Self::Meta,
        _graph: HydroflowGraph,
    ) {
        panic!(".deploy() cannot be called on a CLIRuntimeNode");
    }
}

#[derive(Clone)]
pub struct CLIRuntimeCluster {
    next_port: Rc<RefCell<usize>>,
}

impl Node for CLIRuntimeCluster {
    type Port = String;
    type Meta = ();
    type InstantiateEnv = ();

    fn next_port(&self) -> String {
        let next_send_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_send_port)
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}

    fn instantiate(
        &self,
        _env: &mut Self::InstantiateEnv,
        _meta: &mut Self::Meta,
        _graph: HydroflowGraph,
    ) {
        panic!(".deploy() cannot be called on a CLIRuntimeCluster");
    }
}

impl<'a> ProcessSpec<'a, CLIRuntime> for () {
    fn build(self, _id: usize) -> CLIRuntimeNode {
        CLIRuntimeNode {
            next_port: Rc::new(RefCell::new(0)),
        }
    }
}

impl<'cli> ClusterSpec<'cli, CLIRuntime> for () {
    fn build(self, _id: usize) -> CLIRuntimeCluster {
        CLIRuntimeCluster {
            next_port: Rc::new(RefCell::new(0)),
        }
    }
}
