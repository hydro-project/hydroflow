use hydroflow_plus::*;
use stageleft::*;

pub fn map_reduce<'a, D: Deploy<'a>>(
    flow: &FlowBuilder<'a, D>,
    process_spec: impl ProcessSpec<'a, D>,
    cluster_spec: impl ClusterSpec<'a, D>,
) -> (Process<()>, Cluster<'a, ()>) {
    let process = flow.process(process_spec);
    let cluster = flow.cluster(cluster_spec);

    let words = flow
        .source_iter(&process, q!(vec!["abc", "abc", "xyz", "abc"]))
        .map(q!(|s| s.to_string()));

    let all_ids_vec = cluster.ids();
    let words_partitioned = words
        .enumerate()
        .map(q!(|(i, w)| ((i % all_ids_vec.len()) as u32, w)));

    words_partitioned
        .send_bincode(&cluster)
        .tick_batch()
        .map(q!(|string| (string, ())))
        .fold_keyed(q!(|| 0), q!(|count, _| *count += 1))
        .inspect(q!(|(string, count)| println!(
            "partition count: {} - {}",
            string, count
        )))
        .send_bincode_interleaved(&process)
        .all_ticks()
        .reduce_keyed(q!(|total, count| *total += count))
        .for_each(q!(|(string, count)| println!("{}: {}", string, count)));

    (process, cluster)
}

use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus_cli_integration::{CLIRuntime, HydroflowPlusMeta};

#[stageleft::entry]
pub fn map_reduce_runtime<'a>(
    flow: FlowBuilder<'a, CLIRuntime>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let _ = map_reduce(&flow, (), ());
    flow.with_default_optimize()
        .compile(&cli)
        .with_dynamic_id(q!(cli.meta.subgraph_id))
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use stageleft::RuntimeData;

    #[test]
    fn map_reduce_ir() {
        let builder = hydroflow_plus::FlowBuilder::new();
        let _ = super::map_reduce(&builder, (), ());
        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        for (id, ir) in built
            .with_default_optimize()
            .compile(&RuntimeData::new("FAKE"))
            .hydroflow_ir()
        {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_display_snapshot!(ir.surface_syntax_string());
            });
        }
    }
}
