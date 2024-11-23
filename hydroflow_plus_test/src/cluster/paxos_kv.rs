use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use hydroflow_plus::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use stream::NoOrder;

use super::paxos::{paxos_core, Acceptor, Ballot, Proposer};

pub struct Replica {}

pub trait KvKey: Serialize + DeserializeOwned + Hash + Eq + Clone + Debug {}
impl<K: Serialize + DeserializeOwned + Hash + Eq + Clone + Debug> KvKey for K {}

pub trait KvValue: Serialize + DeserializeOwned + Eq + Clone + Debug {}
impl<V: Serialize + DeserializeOwned + Eq + Clone + Debug> KvValue for V {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct KvPayload<K, V> {
    pub key: K,
    pub value: V,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct SequencedKv<K, V> {
    // Note: Important that seq is the first member of the struct for sorting
    pub seq: usize,
    pub kv: Option<KvPayload<K, V>>,
}

impl<K: KvKey, V: KvValue> Ord for SequencedKv<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.seq.cmp(&other.seq)
    }
}

impl<K: KvKey, V: KvValue> PartialOrd for SequencedKv<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Sets up a linearizable key-value store using Paxos.
///
/// # Safety
/// Notifications for leader election are non-deterministic. When the leader is changing,
/// writes may be dropped by the old leader.
#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "internal paxos code // TODO"
)]
pub unsafe fn paxos_kv<'a, K: KvKey, V: KvValue>(
    proposers: &Cluster<'a, Proposer>,
    acceptors: &Cluster<'a, Acceptor>,
    replicas: &Cluster<'a, Replica>,
    c_to_proposers: Stream<KvPayload<K, V>, Cluster<'a, Proposer>, Unbounded>,
    f: usize,
    i_am_leader_send_timeout: u64,
    i_am_leader_check_timeout: u64,
    i_am_leader_check_timeout_delay_multiplier: usize,
    checkpoint_frequency: usize,
) -> (
    Stream<Ballot, Cluster<'a, Proposer>, Unbounded>,
    Stream<KvPayload<K, V>, Cluster<'a, Replica>, Unbounded>,
) {
    let (r_to_acceptors_checkpoint_complete_cycle, r_to_acceptors_checkpoint) =
        replicas.forward_ref::<Stream<_, _, _>>();

    let (p_to_clients_new_leader_elected, p_to_replicas) = unsafe {
        // SAFETY: Leader election non-determinism and non-deterministic dropping of writes is documented.
        paxos_core(
            proposers,
            acceptors,
            r_to_acceptors_checkpoint.broadcast_bincode(acceptors),
            c_to_proposers,
            f,
            i_am_leader_send_timeout,
            i_am_leader_check_timeout,
            i_am_leader_check_timeout_delay_multiplier,
        )
    };

    let (r_to_acceptors_checkpoint_new, r_new_processed_payloads) = replica(
        replicas,
        p_to_replicas
            .map(q!(|(slot, kv)| SequencedKv { seq: slot, kv }))
            .broadcast_bincode_interleaved(replicas),
        checkpoint_frequency,
    );

    r_to_acceptors_checkpoint_complete_cycle.complete(r_to_acceptors_checkpoint_new);

    (p_to_clients_new_leader_elected, r_new_processed_payloads)
}

// Replicas. All relations for replicas will be prefixed with r. Expects ReplicaPayload on p_to_replicas, outputs a stream of (client address, ReplicaPayload) after processing.
#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
pub fn replica<'a, K: KvKey, V: KvValue>(
    replicas: &Cluster<'a, Replica>,
    p_to_replicas: Stream<SequencedKv<K, V>, Cluster<'a, Replica>, Unbounded, NoOrder>,
    checkpoint_frequency: usize,
) -> (
    Stream<usize, Cluster<'a, Replica>, Unbounded>,
    Stream<KvPayload<K, V>, Cluster<'a, Replica>, Unbounded>,
) {
    let replica_tick = replicas.tick();

    let (r_buffered_payloads_complete_cycle, r_buffered_payloads) = replica_tick.cycle();
    // p_to_replicas.inspect(q!(|payload: ReplicaPayload| println!("Replica received payload: {:?}", payload)));
    let r_sorted_payloads = unsafe {
        // SAFETY: because we fill slots one-by-one, we can safely batch
        // because non-determinism is resolved when we sort by slots
        p_to_replicas
        .timestamped(&replica_tick)
        .tick_batch()
    }
        .union(r_buffered_payloads) // Combine with all payloads that we've received and not processed yet
        .sort();
    // Create a cycle since we'll use this seq before we define it
    let (r_highest_seq_complete_cycle, r_highest_seq) =
        replica_tick.cycle::<Optional<usize, _, _>>();
    // Find highest the sequence number of any payload that can be processed in this tick. This is the payload right before a hole.
    let r_highest_seq_processable_payload = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq.into_singleton())
        .fold(
            q!(|| None),
            q!(|filled_slot, (sorted_payload, highest_seq)| {
                let expected_next_slot = std::cmp::max(
                    filled_slot.map(|v| v + 1).unwrap_or(0),
                    highest_seq.map(|v| v + 1).unwrap_or(0),
                );

                if sorted_payload.seq == expected_next_slot {
                    *filled_slot = Some(sorted_payload.seq);
                }
            }),
        )
        .filter_map(q!(|v| v));
    // Find all payloads that can and cannot be processed in this tick.
    let r_processable_payloads = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq_processable_payload.clone())
        .filter(q!(
            |(sorted_payload, highest_seq)| sorted_payload.seq <= *highest_seq
        ))
        .map(q!(|(sorted_payload, _)| { sorted_payload }));
    let r_new_non_processable_payloads = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq_processable_payload.clone())
        .filter(q!(
            |(sorted_payload, highest_seq)| sorted_payload.seq > *highest_seq
        ))
        .map(q!(|(sorted_payload, _)| { sorted_payload }));
    // Save these, we can process them once the hole has been filled
    r_buffered_payloads_complete_cycle.complete_next_tick(r_new_non_processable_payloads);

    let r_kv_store = r_processable_payloads
        .clone()
        .persist() // Optimization: all_ticks() + fold() = fold<static>, where the state of the previous fold is saved and persisted values are deleted.
        .fold(q!(|| (HashMap::new(), None)), q!(|(kv_store, last_seq), payload| {
            if let Some(kv) = payload.kv {
                kv_store.insert(kv.key, kv.value);
            }

            debug_assert!(payload.seq == (last_seq.map(|s| s + 1).unwrap_or(0)), "Hole in log between seq {:?} and {}", *last_seq, payload.seq);
            *last_seq = Some(payload.seq);
        }));
    // Update the highest seq for the next tick
    let r_new_highest_seq = r_kv_store.filter_map(q!(|(_kv_store, highest_seq)| highest_seq));
    r_highest_seq_complete_cycle.complete_next_tick(r_new_highest_seq.clone());

    // Send checkpoints to the acceptors when we've processed enough payloads
    let (r_checkpointed_seqs_complete_cycle, r_checkpointed_seqs) =
        replica_tick.cycle::<Optional<usize, _, _>>();
    let r_max_checkpointed_seq = r_checkpointed_seqs.persist().max().into_singleton();
    let r_checkpoint_seq_new =
        r_max_checkpointed_seq
            .zip(r_new_highest_seq)
            .filter_map(q!(
                move |(max_checkpointed_seq, new_highest_seq)| if max_checkpointed_seq
                    .map(|m| new_highest_seq - m >= checkpoint_frequency)
                    .unwrap_or(true)
                {
                    Some(new_highest_seq)
                } else {
                    None
                }
            ));
    r_checkpointed_seqs_complete_cycle.complete_next_tick(r_checkpoint_seq_new.clone());

    // Tell clients that the payload has been committed. All ReplicaPayloads contain the client's machine ID (to string) as value.
    let r_to_clients = r_processable_payloads
        .filter_map(q!(|payload| payload.kv))
        .all_ticks();
    (
        r_checkpoint_seq_new.all_ticks().drop_timestamp(),
        r_to_clients.drop_timestamp(),
    )
}
