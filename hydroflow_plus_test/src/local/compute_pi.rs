use std::time::Duration;

use hydroflow_plus::*;
use stageleft::*;

pub fn compute_pi<'a, D: LocalDeploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    process_spec: impl ProcessSpec<'a, D>,
    batch_size: RuntimeData<&'a usize>,
) -> D::Process {
    let process = flow.process(process_spec);

    let trials = flow
        .spin_batch(&process, q!(*batch_size))
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

#[stageleft::entry]
pub fn compute_pi_runtime<'a>(
    flow: FlowBuilder<'a, SingleProcessGraph>,
    batch_size: RuntimeData<&'a usize>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = compute_pi(&flow, (), batch_size);
    flow.with_default_optimize().compile()
}
