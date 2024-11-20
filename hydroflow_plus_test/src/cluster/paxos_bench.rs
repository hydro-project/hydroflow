use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use hydroflow_plus::*;
use stream::NoOrder;

use super::paxos::{Acceptor, Ballot, Proposer};
use super::paxos_kv::{paxos_kv, KvPayload, Replica};

pub struct Client {}

// Important: By convention, all relations that represent booleans either have a single "true" value or nothing.
// This allows us to use the continue_if_exists() and continue_if_empty() operators as if they were if (true) and if (false) statements.
#[expect(clippy::too_many_arguments, reason = "internal paxos code // TODO")]
pub fn paxos_bench<'a>(
    flow: &FlowBuilder<'a>,
    f: usize,
    num_clients_per_node: usize,
    median_latency_window_size: usize, /* How many latencies to keep in the window for calculating the median */
    checkpoint_frequency: usize,       // How many sequence numbers to commit before checkpointing
    i_am_leader_send_timeout: u64,     // How often to heartbeat
    i_am_leader_check_timeout: u64,    // How often to check if heartbeat expired
    i_am_leader_check_timeout_delay_multiplier: usize, /* Initial delay, multiplied by proposer pid, to stagger proposers checking for timeouts */
) -> (
    Cluster<'a, Proposer>,
    Cluster<'a, Acceptor>,
    Cluster<'a, Client>,
    Cluster<'a, Replica>,
) {
    let proposers = flow.cluster::<Proposer>();
    let acceptors = flow.cluster::<Acceptor>();
    let clients = flow.cluster::<Client>();
    let replicas = flow.cluster::<Replica>();

    let (new_leader_elected_complete, new_leader_elected) =
        clients.forward_ref::<Stream<_, _, _, NoOrder>>();

    let client_tick = clients.tick();
    let cur_leader_id = new_leader_elected
        .inspect(q!(|ballot| println!(
            "Client notified that leader was elected: {:?}",
            ballot
        )))
        .max()
        .map(q!(|ballot: Ballot| ballot.proposer_id))
        .latest_tick(&client_tick);

    let leader_changed = cur_leader_id.clone().delta().map(q!(|_| ())).all_ticks();

    bench_client(
        &clients,
        leader_changed,
        |c_to_proposers| {
            let (new_leader_elected, processed_payloads) = paxos_kv(
                &proposers,
                &acceptors,
                &replicas,
                c_to_proposers
                    .tick_batch(&client_tick)
                    .cross_singleton(cur_leader_id)
                    .all_ticks()
                    .map(q!(move |(key, leader_id)| (leader_id, KvPayload {
                        key,
                        // we use our ID as the value and use that so the replica only notifies us
                        value: CLUSTER_SELF_ID
                    })))
                    .send_bincode_interleaved(&proposers)
                    // clients "own" certain keys, so interleaving elements from clients will not affect
                    // the order of writes to the same key
                    .assume_ordering(),
                f,
                i_am_leader_send_timeout,
                i_am_leader_check_timeout,
                i_am_leader_check_timeout_delay_multiplier,
                checkpoint_frequency,
            );

            new_leader_elected_complete
                .complete(new_leader_elected.broadcast_bincode_interleaved(&clients));

            // we only mark a transaction as committed when `f + 1` replicas have committed it
            let (c_pending_quorum_payloads_complete_cycle, c_pending_quorum_payloads) =
                client_tick.cycle::<Stream<_, _, _, NoOrder>>();
            let c_received_payloads = processed_payloads
                .map(q!(|payload| (payload.value, payload)))
                .send_bincode_interleaved(&clients)
                .tick_batch(&client_tick)
                .map(q!(|replica_payload| (replica_payload.key, ())))
                .chain(c_pending_quorum_payloads);
            let c_received_quorum_payloads = c_received_payloads
                .clone()
                .fold_keyed_commutative(
                    q!(|| 0),
                    q!(|curr_count, _sender| {
                        *curr_count += 1; // Assumes the same replica will only send commit once
                    }),
                )
                .filter_map(q!(move |(key, count)| {
                    if count == f + 1 {
                        Some(key)
                    } else {
                        None
                    }
                }));
            let c_new_pending_quorum_payloads =
                c_received_payloads.anti_join(c_received_quorum_payloads.clone());
            c_pending_quorum_payloads_complete_cycle
                .complete_next_tick(c_new_pending_quorum_payloads);

            c_received_quorum_payloads.all_ticks()
        },
        num_clients_per_node,
        median_latency_window_size,
    );

    (proposers, acceptors, clients, replicas)
}

fn bench_client<'a>(
    clients: &Cluster<'a, Client>,
    trigger_restart: Stream<(), Cluster<'a, Client>, Unbounded>,
    transaction_cycle: impl FnOnce(
        Stream<u32, Cluster<'a, Client>, Unbounded>,
    ) -> Stream<u32, Cluster<'a, Client>, Unbounded, NoOrder>,
    num_clients_per_node: usize,
    median_latency_window_size: usize,
) {
    let client_tick = clients.tick();
    // r_to_clients_payload_applied.clone().inspect(q!(|payload: &(u32, ReplicaPayload)| println!("Client received payload: {:?}", payload)));

    // Whenever the leader changes, make all clients send a message
    let restart_this_tick = trigger_restart.tick_batch(&client_tick).last();

    let c_new_payloads_when_restart = restart_this_tick
        .clone()
        .flat_map(q!(move |_| (0..num_clients_per_node).map(move |i| i as u32)));

    let (c_to_proposers_complete_cycle, c_to_proposers) =
        clients.forward_ref::<Stream<_, _, _, NoOrder>>();
    let c_received_quorum_payloads = transaction_cycle(
        c_to_proposers.assume_ordering(), /* we don't send a new write for the same key until the previous one is committed,
                                           * so writes to the same key are ordered */
    )
    .tick_batch(&client_tick);

    // Whenever all replicas confirm that a payload was committed, send another payload
    let c_new_payloads_when_committed = c_received_quorum_payloads.clone();
    c_to_proposers_complete_cycle.complete(
        c_new_payloads_when_restart
            .chain(c_new_payloads_when_committed)
            .all_ticks(),
    );

    // Track statistics
    let (c_timers_complete_cycle, c_timers) =
        client_tick.cycle::<Stream<(usize, SystemTime), _, _, NoOrder>>();
    let c_new_timers_when_leader_elected = restart_this_tick
        .map(q!(|_| SystemTime::now()))
        .flat_map(q!(
            move |now| (0..num_clients_per_node).map(move |virtual_id| (virtual_id, now))
        ));
    let c_updated_timers = c_received_quorum_payloads
        .clone()
        .map(q!(|key| (key as usize, SystemTime::now())));
    let c_new_timers = c_timers
        .clone() // Update c_timers in tick+1 so we can record differences during this tick (to track latency)
        .chain(c_new_timers_when_leader_elected)
        .chain(c_updated_timers.clone())
        .reduce_keyed_commutative(q!(|curr_time, new_time| {
            if new_time > *curr_time {
                *curr_time = new_time;
            }
        }));
    c_timers_complete_cycle.complete_next_tick(c_new_timers);

    let c_stats_output_timer = clients
        .source_interval(q!(Duration::from_secs(1)))
        .tick_batch(&client_tick)
        .first();

    let c_latency_reset = c_stats_output_timer.clone().map(q!(|_| None)).defer_tick();

    let c_latencies = c_timers
        .join(c_updated_timers)
        .map(q!(|(_virtual_id, (prev_time, curr_time))| Some(
            curr_time.duration_since(prev_time).unwrap().as_micros()
        )))
        .chain(c_latency_reset.into_stream())
        .all_ticks()
        .flatten()
        .fold_commutative(
            // Create window with ring buffer using vec + wraparound index
            // TODO: Would be nice if I could use vec![] instead, but that doesn't work in HF+ with RuntimeData *median_latency_window_size
            q!(move || (
                Rc::new(RefCell::new(Vec::<u128>::with_capacity(
                    median_latency_window_size
                ))),
                0usize,
            )),
            q!(move |(latencies, write_index), latency| {
                let mut latencies_mut = latencies.borrow_mut();
                if *write_index < latencies_mut.len() {
                    latencies_mut[*write_index] = latency;
                } else {
                    latencies_mut.push(latency);
                }
                // Increment write index and wrap around
                *write_index = (*write_index + 1) % median_latency_window_size;
            }),
        )
        .map(q!(|(latencies, _)| latencies));

    let c_throughput_new_batch = c_received_quorum_payloads
        .clone()
        .count()
        .continue_unless(c_stats_output_timer.clone())
        .map(q!(|batch_size| (batch_size, false)));

    let c_throughput_reset = c_stats_output_timer
        .clone()
        .map(q!(|_| (0, true)))
        .defer_tick();

    let c_throughput = c_throughput_new_batch
        .union(c_throughput_reset)
        .all_ticks()
        .fold(
            q!(|| 0),
            q!(|total, (batch_size, reset)| {
                if reset {
                    *total = 0;
                } else {
                    *total += batch_size;
                }
            }),
        );

    c_latencies
        .zip(c_throughput)
        .latest_tick(&client_tick)
        .continue_if(c_stats_output_timer)
        .all_ticks()
        .for_each(q!(move |(latencies, throughput)| {
            let mut latencies_mut = latencies.borrow_mut();
            if latencies_mut.len() > 0 {
                let middle_idx = latencies_mut.len() / 2;
                let (_, median, _) = latencies_mut.select_nth_unstable(middle_idx);
                println!("Median latency: {}ms", (*median) as f64 / 1000.0);
            }

            println!("Throughput: {} requests/s", throughput);
        }));
    // End track statistics
}

#[cfg(test)]
mod tests {
    use hydroflow_plus::deploy::DeployRuntime;
    use stageleft::RuntimeData;

    #[test]
    fn paxos_ir() {
        let builder = hydroflow_plus::FlowBuilder::new();
        let _ = super::paxos_bench(&builder, 1, 1, 1, 1, 1, 1, 1);
        let built = builder.with_default_optimize::<DeployRuntime>();

        hydroflow_plus::ir::dbg_dedup_tee(|| {
            insta::assert_debug_snapshot!(built.ir());
        });

        let _ = built.compile(&RuntimeData::new("FAKE"));
    }
}
