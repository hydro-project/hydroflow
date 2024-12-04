pub mod compute_pi;
pub mod many_to_many;
pub mod map_reduce;
pub mod paxos;
pub mod paxos_bench;
pub mod paxos_kv;
pub mod quorum;
pub mod request_response;
pub mod simple_cluster;
pub mod two_pc;

#[doc(hidden)]
#[stageleft::runtime]
pub mod docs {
    #[doc = include_str!("../../../docs/docs/hydroflow_plus/consistency.mdx")]
    mod consistency {}
}
