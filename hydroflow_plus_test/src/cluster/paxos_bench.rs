use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use hydroflow_plus::*;
use stageleft::*;

use super::paxos::{Acceptor, Proposer};
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
        clients.forward_ref::<Stream<_, _, _>>();

    bench_client(
        &clients,
        new_leader_elected,
        |c_to_proposers| {
            let (new_leader_elected, processed_payloads) = paxos_kv(
                &proposers,
                &acceptors,
                &replicas,
                c_to_proposers.send_bincode_interleaved(&proposers),
                f,
                i_am_leader_send_timeout,
                i_am_leader_check_timeout,
                i_am_leader_check_timeout_delay_multiplier,
                checkpoint_frequency,
            );

            new_leader_elected_complete.complete(
                new_leader_elected
                    .broadcast_bincode(&clients)
                    .map(q!(|(leader_id, _)| leader_id)),
            );
            processed_payloads
                .map(q!(|payload| (payload.value, payload)))
                .send_bincode(&clients)
        },
        num_clients_per_node,
        median_latency_window_size,
        f,
    );

    (proposers, acceptors, clients, replicas)
}

// Clients. All relations for clients will be prefixed with c. All ClientPayloads will contain the virtual client number as key and the client's machine ID (to string) as value. Expects p_to_clients_leader_elected containing Ballots whenever the leader is elected, and r_to_clients_payload_applied containing ReplicaPayloads whenever a payload is committed. Outputs (leader address, ClientPayload) when a new leader is elected or when the previous payload is committed.
fn bench_client<'a>(
    clients: &Cluster<'a, Client>,
    p_to_clients_leader_elected: Stream<ClusterId<Proposer>, Cluster<'a, Client>, Unbounded>,
    transaction_cycle: impl FnOnce(
        Stream<
            (ClusterId<Proposer>, KvPayload<u32, ClusterId<Client>>),
            Cluster<'a, Client>,
            Unbounded,
        >,
    ) -> Stream<
        (ClusterId<Replica>, KvPayload<u32, ClusterId<Client>>),
        Cluster<'a, Client>,
        Unbounded,
    >,
    num_clients_per_node: usize,
    median_latency_window_size: usize,
    f: usize,
) {
    let client_tick = clients.tick();
    let c_id = clients.self_id();
    // r_to_clients_payload_applied.clone().inspect(q!(|payload: &(u32, ReplicaPayload)| println!("Client received payload: {:?}", payload)));
    // Only keep the latest leader
    let current_leader = p_to_clients_leader_elected
        .inspect(q!(|ballot| println!(
            "Client notified that leader was elected: {:?}",
            ballot
        )))
        .max();
    let c_new_leader_ballot = current_leader.clone().latest_tick(&client_tick).delta();
    // Whenever the leader changes, make all clients send a message
    let c_new_payloads_when_leader_elected =
        c_new_leader_ballot
            .clone()
            .flat_map(q!(move |leader_ballot| (0..num_clients_per_node).map(
                move |i| (
                    leader_ballot,
                    KvPayload {
                        key: i as u32,
                        value: c_id
                    }
                )
            )));

    let (c_to_proposers_complete_cycle, c_to_proposers) = clients.forward_ref();
    let transaction_results = transaction_cycle(c_to_proposers);

    // Whenever replicas confirm that a payload was committed, collected it and wait for a quorum
    let (c_pending_quorum_payloads_complete_cycle, c_pending_quorum_payloads) = client_tick.cycle();
    let c_received_payloads = transaction_results
        .tick_batch(&client_tick)
        .map(q!(|(sender, replica_payload)| (
            replica_payload.key,
            sender
        )))
        .union(c_pending_quorum_payloads);
    let c_received_quorum_payloads = c_received_payloads
        .clone()
        .fold_keyed(
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
    c_pending_quorum_payloads_complete_cycle.complete_next_tick(c_new_pending_quorum_payloads);
    // Whenever all replicas confirm that a payload was committed, send another payload
    let c_new_payloads_when_committed = c_received_quorum_payloads
        .clone()
        .cross_singleton(current_leader.clone().latest_tick(&client_tick))
        .map(q!(move |(key, cur_leader)| (
            cur_leader,
            KvPayload { key, value: c_id }
        )));
    c_to_proposers_complete_cycle.complete(
        c_new_payloads_when_leader_elected
            .union(c_new_payloads_when_committed)
            .all_ticks(),
    );

    // Track statistics
    let (c_timers_complete_cycle, c_timers) =
        client_tick.cycle::<Stream<(usize, SystemTime), _, _>>();
    let c_new_timers_when_leader_elected = c_new_leader_ballot
        .map(q!(|_| SystemTime::now()))
        .flat_map(q!(
            move |now| (0..num_clients_per_node).map(move |virtual_id| (virtual_id, now))
        ));
    let c_updated_timers = c_received_quorum_payloads
        .clone()
        .map(q!(|key| (key as usize, SystemTime::now())));
    let c_new_timers = c_timers
        .clone() // Update c_timers in tick+1 so we can record differences during this tick (to track latency)
        .union(c_new_timers_when_leader_elected)
        .union(c_updated_timers.clone())
        .reduce_keyed(q!(|curr_time, new_time| {
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
        .union(c_latency_reset.into_stream())
        .all_ticks()
        .flatten()
        .fold(
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
        let built = builder.with_default_optimize();

        hydroflow_plus::ir::dbg_dedup_tee(|| {
            insta::assert_debug_snapshot!(built.ir());
        });

        let _ = built.compile::<DeployRuntime>(&RuntimeData::new("FAKE"));
    }
}
