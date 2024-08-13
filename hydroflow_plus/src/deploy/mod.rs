use hydroflow_lang::graph::HydroflowGraph;
use stageleft::Quoted;

pub mod graphs;
pub use graphs::*;

pub trait LocalDeploy<'a> {
    type ClusterId: Clone + 'static;
    type Process: Node<Meta = Self::Meta>;
    type Cluster: Node<Meta = Self::Meta>;
    type Meta: Default;
    type GraphId;
}

pub trait Deploy<'a> {
    /// Type of ID used to identify individual members of a cluster.
    type ClusterId: Clone + 'static;
    type InstantiateEnv;
    type CompileEnv;

    type Process: Node<Meta = Self::Meta, InstantiateEnv = Self::InstantiateEnv> + Clone;
    type Cluster: Node<Meta = Self::Meta, InstantiateEnv = Self::InstantiateEnv> + Clone;
    type ProcessPort;
    type ClusterPort;
    type Meta: Default;

    /// Type of ID used to switch between different subgraphs at runtime.
    type GraphId;

    fn allocate_process_port(process: &Self::Process) -> Self::ProcessPort;
    fn allocate_cluster_port(cluster: &Self::Cluster) -> Self::ClusterPort;

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

    fn cluster_ids(
        env: &Self::CompileEnv,
        of_cluster: usize,
    ) -> impl Quoted<'a, &'a Vec<Self::ClusterId>> + Copy + 'a;
    fn cluster_self_id(env: &Self::CompileEnv) -> impl Quoted<'a, Self::ClusterId> + Copy + 'a;
}

impl<
        'a,
        Cid: Clone + 'static,
        T: Deploy<'a, ClusterId = Cid, Process = N, Cluster = C, Meta = M, GraphId = R>,
        N: Node<Meta = M>,
        C: Node<Meta = M>,
        M: Default,
        R,
    > LocalDeploy<'a> for T
{
    type ClusterId = Cid;
    type Process = N;
    type Cluster = C;
    type Meta = M;
    type GraphId = R;
}

pub trait ProcessSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(self, id: usize) -> D::Process;
}

pub trait ClusterSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(self, id: usize) -> D::Cluster;
}

pub trait Node {
    type Port;
    type Meta;
    type InstantiateEnv;

    fn id(&self) -> usize;

    fn next_port(&self) -> Self::Port;

    fn update_meta(&mut self, meta: &Self::Meta);

    fn instantiate(
        &self,
        env: &mut Self::InstantiateEnv,
        meta: &mut Self::Meta,
        graph: HydroflowGraph,
    );
}
