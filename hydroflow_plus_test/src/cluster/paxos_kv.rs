use std::collections::HashMap;

use hydroflow_plus::*;
use serde::{Deserialize, Serialize};
use stageleft::*;

use super::paxos::{paxos_core, Acceptor, PaxosPayload, Proposer};

pub struct Replica {}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct KvPayload {
    pub key: u32,
    pub value: String,
}

impl Default for KvPayload {
    fn default() -> Self {
        Self {
            key: 0,
            value: "".to_string(),
        }
    }
}

impl PaxosPayload for KvPayload {}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SequencedKv {
    // Note: Important that seq is the first member of the struct for sorting
    pub seq: i32,
    pub kv: KvPayload,
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "internal paxos code // TODO"
)]
pub fn paxos_kv<'a>(
    proposers: &Cluster<'a, Proposer>,
    acceptors: &Cluster<'a, Acceptor>,
    replicas: &Cluster<'a, Replica>,
    c_to_proposers: Stream<KvPayload, Unbounded, NoTick, Cluster<'a, Proposer>>,
    f: usize,
    i_am_leader_send_timeout: u64,
    i_am_leader_check_timeout: u64,
    i_am_leader_check_timeout_delay_multiplier: usize,
    checkpoint_frequency: usize,
) -> (
    Stream<(), Unbounded, NoTick, Cluster<'a, Proposer>>,
    Stream<SequencedKv, Unbounded, NoTick, Cluster<'a, Replica>>,
) {
    let (r_to_acceptors_checkpoint_complete_cycle, r_to_acceptors_checkpoint) =
        replicas.forward_ref::<Stream<_, _, _, _>>();

    let (p_to_clients_new_leader_elected, p_to_replicas) = paxos_core(
        proposers,
        acceptors,
        r_to_acceptors_checkpoint.broadcast_bincode(acceptors),
        c_to_proposers,
        f,
        i_am_leader_send_timeout,
        i_am_leader_check_timeout,
        i_am_leader_check_timeout_delay_multiplier,
    );

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
pub fn replica<'a>(
    replicas: &Cluster<'a, Replica>,
    p_to_replicas: Stream<SequencedKv, Unbounded, NoTick, Cluster<'a, Replica>>,
    checkpoint_frequency: usize,
) -> (
    Stream<i32, Unbounded, NoTick, Cluster<'a, Replica>>,
    Stream<SequencedKv, Unbounded, NoTick, Cluster<'a, Replica>>,
) {
    let (r_buffered_payloads_complete_cycle, r_buffered_payloads) = replicas.tick_cycle();
    // p_to_replicas.inspect(q!(|payload: ReplicaPayload| println!("Replica received payload: {:?}", payload)));
    let r_sorted_payloads = p_to_replicas
        .clone()
        .tick_batch()
        .union(r_buffered_payloads) // Combine with all payloads that we've received and not processed yet
        .sort();
    // Create a cycle since we'll use this seq before we define it
    let (r_highest_seq_complete_cycle, r_highest_seq) =
        replicas.tick_cycle::<Optional<i32, _, _, _>>();
    let empty_slot = replicas.singleton_first_tick(q!(-1));
    // Either the max sequence number executed so far or -1. Need to union otherwise r_highest_seq is empty and joins with it will fail
    let r_highest_seq_with_default = r_highest_seq.union(empty_slot);
    // Find highest the sequence number of any payload that can be processed in this tick. This is the payload right before a hole.
    let r_highest_seq_processable_payload = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq_with_default)
        .fold(
            q!(|| -1),
            q!(|filled_slot, (sorted_payload, highest_seq)| {
                // Note: This function only works if the input is sorted on seq.
                let next_slot = std::cmp::max(*filled_slot, highest_seq);

                *filled_slot = if sorted_payload.seq == next_slot + 1 {
                    sorted_payload.seq
                } else {
                    *filled_slot
                };
            }),
        );
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
        .fold(q!(|| (HashMap::<u32, String>::new(), -1)), q!(|state, payload| {
            let kv_store = &mut state.0;
            let last_seq = &mut state.1;
            kv_store.insert(payload.kv.key, payload.kv.value);
            debug_assert!(payload.seq == *last_seq + 1, "Hole in log between seq {} and {}", *last_seq, payload.seq);
            *last_seq = payload.seq;
            // println!("Replica kv store: {:?}", kv_store);
        }));
    // Update the highest seq for the next tick
    let r_new_highest_seq = r_kv_store.map(q!(|(_kv_store, highest_seq)| highest_seq));
    r_highest_seq_complete_cycle.complete_next_tick(r_new_highest_seq.clone().into());

    // Send checkpoints to the acceptors when we've processed enough payloads
    let (r_checkpointed_seqs_complete_cycle, r_checkpointed_seqs) =
        replicas.tick_cycle::<Optional<i32, _, _, _>>();
    let r_max_checkpointed_seq = r_checkpointed_seqs
        .persist()
        .max()
        .unwrap_or(replicas.singleton(q!(-1)).latest_tick());
    let r_checkpoint_seq_new = r_max_checkpointed_seq
        .cross_singleton(r_new_highest_seq)
        .filter_map(q!(
            move |(max_checkpointed_seq, new_highest_seq)| if new_highest_seq - max_checkpointed_seq
                >= checkpoint_frequency as i32
            {
                Some(new_highest_seq)
            } else {
                None
            }
        ));
    r_checkpointed_seqs_complete_cycle.complete_next_tick(r_checkpoint_seq_new.clone());

    // Tell clients that the payload has been committed. All ReplicaPayloads contain the client's machine ID (to string) as value.
    let r_to_clients = p_to_replicas;
    (r_checkpoint_seq_new.all_ticks(), r_to_clients)
}
