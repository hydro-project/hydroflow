use std::cell::RefCell;
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use hydroflow_plus::profiler::profiling;
use hydroflow_plus::*;
use stageleft::*;

pub fn compute_pi<'a, D: Deploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    process_spec: impl ProcessSpec<'a, D>,
    cluster_spec: impl ClusterSpec<'a, D>,
    batch_size: RuntimeData<&'a usize>,
) -> D::Process {
    let cluster = flow.cluster(cluster_spec);
    let process = flow.process(process_spec);

    let trials = flow
        .spin_batch(&cluster, q!(*batch_size))
        .map(q!(|_| rand::random::<(f64, f64)>()))
        .map(q!(|(x, y)| x * x + y * y < 1.0))
        .fold(
            q!(|| (0u64, 0u64)),
            q!(|(inside, total), sample_inside| {
                if sample_inside {
                    *inside += 1;
                }

                *total += 1;
            }),
        );

    trials
        .send_bincode_interleaved(&process)
        .all_ticks()
        .reduce(q!(|(inside, total), (inside_batch, total_batch)| {
            *inside += inside_batch;
            *total += total_batch;
        }))
        .sample_every(q!(Duration::from_secs(1)))
        .for_each(q!(|(inside, total)| {
            println!(
                "pi: {} ({} trials)",
                4.0 * inside as f64 / total as f64,
                total
            );
        }));

    process
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn compute_pi_runtime<'a>(
    flow: FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    batch_size: RuntimeData<&'a usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = compute_pi(&flow, &cli, &cli, batch_size);
    flow.with_default_optimize()
        .compile()
        .with_dynamic_id(q!(cli.meta.subgraph_id))
}

#[stageleft::entry]
pub fn cardinality_compute_pi_runtime<'a>(
    flow: FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
    batch_size: RuntimeData<&'a usize>,
    counters: RuntimeData<&'a RefCell<Vec<u64>>>,
    counter_queue: RuntimeData<&'a RefCell<UnboundedSender<(usize, u64)>>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = compute_pi(&flow, &cli, &cli, batch_size);
    let runtime_context = flow.runtime_context();
    flow.optimize_with(|ir| profiling(ir, runtime_context, counters, counter_queue))
        .compile()
        .with_dynamic_id(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use stageleft::RuntimeData;

    #[test]
    fn compute_pi_ir() {
        let builder = hydroflow_plus::FlowBuilder::new();
        let _ = super::compute_pi(
            &builder,
            &RuntimeData::new("FAKE"),
            &RuntimeData::new("FAKE"),
            RuntimeData::new("FAKE"),
        );
        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        for (id, ir) in built.compile().hydroflow_ir() {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_display_snapshot!(ir.surface_syntax_string());
            });
        }
    }
}
