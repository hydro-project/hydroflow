use dyn_clone::DynClone;
use hydroflow_lang::graph::HydroflowGraph;
use stageleft::Quoted;

pub mod graphs;
pub use graphs::*;

pub mod network;
pub use network::*;

pub trait LocalDeploy<'a> {
    type ClusterId: Clone + 'static;
    type Process: Location<Meta = Self::Meta> + Clone;
    type Cluster: Location<Meta = Self::Meta> + Cluster<'a, Id = Self::ClusterId> + Clone;
    type Meta: Default;
    type GraphId;
}

pub trait Deploy<'a> {
    /// Type of ID used to identify individual members of a cluster.
    type ClusterId: Clone + 'static;
    type InstantiateEnv;

    type Process: Location<Meta = Self::Meta, Port = Self::ProcessPort, InstantiateEnv = Self::InstantiateEnv>
        + HfSendOneToOne<Self::Process>
        + HfSendOneToMany<Self::Cluster, Self::ClusterId>
        + Clone
        + 'a;
    type Cluster: Location<Meta = Self::Meta, Port = Self::ClusterPort, InstantiateEnv = Self::InstantiateEnv>
        + HfSendManyToOne<Self::Process, Self::ClusterId>
        + HfSendManyToMany<Self::Cluster, Self::ClusterId>
        + Cluster<'a, Id = Self::ClusterId>
        + Clone
        + 'a;
    type ProcessPort;
    type ClusterPort;
    type Meta: Default;

    /// Type of ID used to switch between different subgraphs at runtime.
    type GraphId;
}

impl<
        'a,
        Cid: Clone + 'static,
        T: Deploy<'a, ClusterId = Cid, Process = N, Cluster = C, Meta = M, GraphId = R>,
        N: Location<Meta = M> + HfSendOneToOne<N> + HfSendOneToMany<C, Cid> + Clone,
        C: Location<Meta = M>
            + HfSendManyToOne<N, Cid>
            + HfSendManyToMany<C, Cid>
            + Cluster<'a, Id = Cid>
            + Clone,
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
    fn build(&self, id: usize) -> D::Process;
}

pub trait ClusterSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(&self, id: usize) -> D::Cluster;
}

pub trait Location: DynClone {
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

pub trait Cluster<'a>: Location {
    type Id: 'static;

    fn ids<'b>(&'b self) -> impl Quoted<'a, &'a Vec<Self::Id>> + Copy + 'a;

    fn self_id<'b>(&'b self) -> impl Quoted<'a, Self::Id> + Copy + 'a;
}
