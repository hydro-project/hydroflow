use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use stageleft::{Quoted, RuntimeData};

use super::HydroflowPlusMeta;
use crate::deploy::{ClusterSpec, Deploy, ExternalSpec, Node, ProcessSpec, RegisterPort};
use crate::lang::graph::HydroflowGraph;
use crate::util::deploy::DeployPorts;

pub struct DeployRuntime {}

impl<'a> Deploy<'a> for DeployRuntime {
    type InstantiateEnv = ();
    type CompileEnv = RuntimeData<&'a DeployPorts<HydroflowPlusMeta>>;
    type Process = DeployRuntimeNode;
    type Cluster = DeployRuntimeCluster;
    type ExternalProcess = DeployRuntimeNode;
    type Port = String;
    type ExternalRawPort = ();
    type Meta = ();
    type GraphId = usize;

    fn has_trivial_node() -> bool {
        true
    }

    fn trivial_process(_id: usize) -> Self::Process {
        DeployRuntimeNode {
            next_port: Rc::new(RefCell::new(0)),
        }
    }

    fn trivail_cluster(_id: usize) -> Self::Cluster {
        DeployRuntimeCluster {
            next_port: Rc::new(RefCell::new(0)),
        }
    }

    fn allocate_process_port(process: &Self::Process) -> Self::Port {
        process.next_port()
    }

    fn allocate_cluster_port(cluster: &Self::Cluster) -> Self::Port {
        cluster.next_port()
    }

    fn allocate_external_port(_external: &Self::ExternalProcess) -> Self::Port {
        panic!();
    }

    fn o2o_sink_source(
        env: &Self::CompileEnv,
        _p1: &Self::Process,
        p1_port: &Self::Port,
        _p2: &Self::Process,
        p2_port: &Self::Port,
    ) -> (syn::Expr, syn::Expr) {
        super::deploy_runtime::deploy_o2o(*env, p1_port.as_str(), p2_port.as_str())
    }

    fn o2o_connect(
        _p1: &Self::Process,
        _p1_port: &Self::Port,
        _p2: &Self::Process,
        _p2_port: &Self::Port,
    ) -> Box<dyn FnOnce()> {
        panic!()
    }

    fn o2m_sink_source(
        env: &Self::CompileEnv,
        _p1: &Self::Process,
        p1_port: &Self::Port,
        _c2: &Self::Cluster,
        c2_port: &Self::Port,
    ) -> (syn::Expr, syn::Expr) {
        super::deploy_runtime::deploy_o2m(*env, p1_port.as_str(), c2_port.as_str())
    }

    fn o2m_connect(
        _p1: &Self::Process,
        _p1_port: &Self::Port,
        _c2: &Self::Cluster,
        _c2_port: &Self::Port,
    ) -> Box<dyn FnOnce()> {
        panic!()
    }

    fn m2o_sink_source(
        env: &Self::CompileEnv,
        _c1: &Self::Cluster,
        c1_port: &Self::Port,
        _p2: &Self::Process,
        p2_port: &Self::Port,
    ) -> (syn::Expr, syn::Expr) {
        super::deploy_runtime::deploy_m2o(*env, c1_port.as_str(), p2_port.as_str())
    }

    fn m2o_connect(
        _c1: &Self::Cluster,
        _c1_port: &Self::Port,
        _p2: &Self::Process,
        _p2_port: &Self::Port,
    ) -> Box<dyn FnOnce()> {
        panic!()
    }

    fn m2m_sink_source(
        env: &Self::CompileEnv,
        _c1: &Self::Cluster,
        c1_port: &Self::Port,
        _c2: &Self::Cluster,
        c2_port: &Self::Port,
    ) -> (syn::Expr, syn::Expr) {
        super::deploy_runtime::deploy_m2m(*env, c1_port.as_str(), c2_port.as_str())
    }

    fn m2m_connect(
        _c1: &Self::Cluster,
        _c1_port: &Self::Port,
        _c2: &Self::Cluster,
        _c2_port: &Self::Port,
    ) -> Box<dyn FnOnce()> {
        panic!()
    }

    fn e2o_source(
        _compile_env: &Self::CompileEnv,
        _p1: &Self::ExternalProcess,
        _p1_port: &Self::Port,
        _p2: &Self::Process,
        _p2_port: &Self::Port,
    ) -> syn::Expr {
        panic!()
    }

    fn e2o_connect(
        _p1: &Self::ExternalProcess,
        _p1_port: &Self::Port,
        _p2: &Self::Process,
        _p2_port: &Self::Port,
    ) -> Box<dyn FnOnce()> {
        panic!()
    }

    fn o2e_sink(
        _compile_env: &Self::CompileEnv,
        _p1: &Self::Process,
        _p1_port: &Self::Port,
        _p2: &Self::ExternalProcess,
        _p2_port: &Self::Port,
    ) -> syn::Expr {
        panic!()
    }

    fn o2e_connect(
        _p1: &Self::Process,
        _p1_port: &Self::Port,
        _p2: &Self::ExternalProcess,
        _p2_port: &Self::Port,
    ) -> Box<dyn FnOnce()> {
        panic!()
    }

    fn cluster_ids(
        env: &Self::CompileEnv,
        of_cluster: usize,
    ) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        super::deploy_runtime::cluster_members(*env, of_cluster)
    }

    fn cluster_self_id(env: &Self::CompileEnv) -> impl Quoted<'a, u32> + Copy + 'a {
        super::deploy_runtime::cluster_self_id(*env)
    }
}

#[derive(Clone)]
pub struct DeployRuntimeNode {
    next_port: Rc<RefCell<usize>>,
}

impl<'a> RegisterPort<'a, DeployRuntime> for DeployRuntimeNode {
    fn register(&self, _key: usize, _port: <DeployRuntime as Deploy>::Port) {
        panic!()
    }

    fn raw_port(&self, _key: usize) -> <DeployRuntime as Deploy>::ExternalRawPort {
        panic!()
    }

    #[expect(
        clippy::manual_async_fn,
        reason = "buggy Clippy lint for lifetime bounds"
    )]
    fn as_bytes_sink(
        &self,
        _key: usize,
    ) -> impl std::future::Future<
        Output = Pin<Box<dyn crate::futures::Sink<crate::bytes::Bytes, Error = std::io::Error>>>,
    > + 'a {
        async { panic!() }
    }

    #[expect(
        clippy::manual_async_fn,
        reason = "buggy Clippy lint for lifetime bounds"
    )]
    fn as_bincode_sink<T: serde::Serialize + 'static>(
        &self,
        _key: usize,
    ) -> impl std::future::Future<
        Output = Pin<Box<dyn crate::futures::Sink<T, Error = std::io::Error>>>,
    > + 'a {
        async { panic!() }
    }

    #[expect(
        clippy::manual_async_fn,
        reason = "buggy Clippy lint for lifetime bounds"
    )]
    fn as_bytes_source(
        &self,
        _key: usize,
    ) -> impl std::future::Future<
        Output = Pin<Box<dyn hydroflow::futures::Stream<Item = hydroflow::bytes::Bytes>>>,
    > + 'a {
        async { panic!() }
    }

    #[expect(
        clippy::manual_async_fn,
        reason = "buggy Clippy lint for lifetime bounds"
    )]
    fn as_bincode_source<T: serde::de::DeserializeOwned + 'static>(
        &self,
        _key: usize,
    ) -> impl std::future::Future<Output = Pin<Box<dyn hydroflow::futures::Stream<Item = T>>>> + 'a
    {
        async { panic!() }
    }
}

impl Node for DeployRuntimeNode {
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
        _extra_stmts: Vec<syn::Stmt>,
    ) {
        panic!(".deploy() cannot be called on a DeployRuntimeNode");
    }
}

#[derive(Clone)]
pub struct DeployRuntimeCluster {
    next_port: Rc<RefCell<usize>>,
}

impl Node for DeployRuntimeCluster {
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
        _extra_stmts: Vec<syn::Stmt>,
    ) {
        panic!(".deploy() cannot be called on a DeployRuntimeCluster");
    }
}

impl<'a> ProcessSpec<'a, DeployRuntime> for () {
    fn build(self, _id: usize, _name_hint: &str) -> DeployRuntimeNode {
        DeployRuntimeNode {
            next_port: Rc::new(RefCell::new(0)),
        }
    }
}

impl<'cli> ClusterSpec<'cli, DeployRuntime> for () {
    fn build(self, _id: usize, _name_hint: &str) -> DeployRuntimeCluster {
        DeployRuntimeCluster {
            next_port: Rc::new(RefCell::new(0)),
        }
    }
}

impl<'cli> ExternalSpec<'cli, DeployRuntime> for () {
    fn build(self, _id: usize, _name_hint: &str) -> DeployRuntimeNode {
        panic!()
    }
}
