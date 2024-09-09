use std::future::Future;
use std::io::Error;
use std::pin::Pin;

use hydroflow::bytes::Bytes;
use hydroflow::futures::Sink;
use hydroflow_lang::graph::HydroflowGraph;
use serde::Serialize;
use stageleft::Quoted;

pub mod graphs;
pub use graphs::*;

pub trait LocalDeploy<'a> {
    type Process: Node<Meta = Self::Meta>;
    type Cluster: Node<Meta = Self::Meta>;
    type ExternalProcess: Node<Meta = Self::Meta>;
    type Meta: Default;
    type GraphId;

    fn has_trivial_node() -> bool {
        false
    }

    fn trivial_process(_id: usize) -> Self::Process {
        panic!("No trivial process")
    }

    fn trivial_cluster(_id: usize) -> Self::Cluster {
        panic!("No trivial cluster")
    }
}

pub trait Deploy<'a> {
    type InstantiateEnv;
    type CompileEnv;

    type Process: Node<Meta = Self::Meta, InstantiateEnv = Self::InstantiateEnv> + Clone;
    type Cluster: Node<Meta = Self::Meta, InstantiateEnv = Self::InstantiateEnv> + Clone;
    type ExternalProcess: Node<Meta = Self::Meta, InstantiateEnv = Self::InstantiateEnv>
        + RegisterPort<'a, Self>
        + Clone;
    type ProcessPort;
    type ClusterPort;
    type ExternalPort: Clone;
    type ExternalGotPort;
    type Meta: Default;

    /// Type of ID used to switch between different subgraphs at runtime.
    type GraphId;

    fn has_trivial_node() -> bool {
        false
    }

    fn trivial_process(_id: usize) -> Self::Process {
        panic!("No trivial process")
    }

    fn trivail_cluster(_id: usize) -> Self::Cluster {
        panic!("No trivial cluster")
    }

    fn allocate_process_port(process: &Self::Process) -> Self::ProcessPort;
    fn allocate_cluster_port(cluster: &Self::Cluster) -> Self::ClusterPort;
    fn allocate_external_port(external: &Self::ExternalProcess) -> Self::ExternalPort;

    fn o2o_sink_source(
        compile_env: &Self::CompileEnv,
        p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> (syn::Expr, syn::Expr);
    fn o2o_connect(
        p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    );

    fn o2m_sink_source(
        compile_env: &Self::CompileEnv,
        p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) -> (syn::Expr, syn::Expr);
    fn o2m_connect(
        p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    );

    fn m2o_sink_source(
        compile_env: &Self::CompileEnv,
        c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> (syn::Expr, syn::Expr);
    fn m2o_connect(
        c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    );

    fn m2m_sink_source(
        compile_env: &Self::CompileEnv,
        c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) -> (syn::Expr, syn::Expr);
    fn m2m_connect(
        c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    );

    fn e2o_source(
        compile_env: &Self::CompileEnv,
        p1: &Self::ExternalProcess,
        p1_port: &Self::ExternalPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> syn::Expr;

    fn e2o_connect(
        p1: &Self::ExternalProcess,
        p1_port: &Self::ExternalPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    );

    fn cluster_ids(
        env: &Self::CompileEnv,
        of_cluster: usize,
    ) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a;
    fn cluster_self_id(env: &Self::CompileEnv) -> impl Quoted<'a, u32> + Copy + 'a;
}

impl<
        'a,
        T: Deploy<'a, Process = N, Cluster = C, ExternalProcess = E, Meta = M, GraphId = R>,
        N: Node<Meta = M>,
        C: Node<Meta = M>,
        E: Node<Meta = M>,
        M: Default,
        R,
    > LocalDeploy<'a> for T
{
    type Process = N;
    type Cluster = C;
    type ExternalProcess = E;
    type Meta = M;
    type GraphId = R;

    fn has_trivial_node() -> bool {
        <T as Deploy<'a>>::has_trivial_node()
    }

    fn trivial_process(id: usize) -> Self::Process {
        <T as Deploy<'a>>::trivial_process(id)
    }

    fn trivial_cluster(id: usize) -> Self::Cluster {
        <T as Deploy<'a>>::trivail_cluster(id)
    }
}

pub trait ProcessSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(self, id: usize, name_hint: &str) -> D::Process;
}

pub trait ClusterSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(self, id: usize, name_hint: &str) -> D::Cluster;
}

pub trait ExternalSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(self, id: usize, name_hint: &str) -> D::ExternalProcess;
}

pub trait Node {
    type Port;
    type Meta;
    type InstantiateEnv;

    fn next_port(&self) -> Self::Port;

    fn update_meta(&mut self, meta: &Self::Meta);

    fn instantiate(
        &self,
        env: &mut Self::InstantiateEnv,
        meta: &mut Self::Meta,
        graph: HydroflowGraph,
        extra_stmts: Vec<syn::Stmt>,
    );
}

pub trait RegisterPort<'a, D: Deploy<'a> + ?Sized> {
    fn register(&self, key: usize, port: D::ExternalPort);
    fn get_port(&self, key: usize) -> D::ExternalGotPort;
    fn as_bytes_sink(
        &self,
        key: usize,
    ) -> impl Future<Output = Pin<Box<dyn Sink<Bytes, Error = Error>>>> + 'a;
    fn as_bincode_sink<T: Serialize + 'static>(
        &self,
        key: usize,
    ) -> impl Future<Output = Pin<Box<dyn Sink<T, Error = Error>>>> + 'a;
}
