use hydroflow_plus::*;
use stageleft::*;

pub struct Leader {}
pub struct Worker {}

pub fn map_reduce(flow: &FlowBuilder) -> (Process<Leader>, Cluster<Worker>) {
    let process = flow.process();
    let cluster = flow.cluster();

    let words = flow
        .source_iter(&process, q!(vec!["abc", "abc", "xyz", "abc"]))
        .map(q!(|s| s.to_string()));

    let all_ids_vec = flow.cluster_members(&cluster);
    let words_partitioned = words
        .tick_batch()
        .enumerate()
        .map(q!(|(i, w)| ((i % all_ids_vec.len()) as u32, w)))
        .all_ticks();

    words_partitioned
        .send_bincode(&cluster)
        .map(q!(|string| (string, ())))
        .tick_batch()
        .fold_keyed(q!(|| 0), q!(|count, _| *count += 1))
        .inspect(q!(|(string, count)| println!(
            "partition count: {} - {}",
            string, count
        )))
        .all_ticks()
        .send_bincode_interleaved(&process)
        .tick_batch()
        .persist()
        .reduce_keyed(q!(|total, count| *total += count))
        .all_ticks()
        .for_each(q!(|(string, count)| println!("{}: {}", string, count)));

    (process, cluster)
}

#[cfg(test)]
mod tests {
    use hydroflow_plus::deploy::DeployRuntime;
    use stageleft::RuntimeData;

    #[test]
    fn map_reduce_ir() {
        let builder = hydroflow_plus::FlowBuilder::new();
        let _ = super::map_reduce(&builder);
        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        for (id, ir) in built
            .compile::<DeployRuntime>(&RuntimeData::new("FAKE"))
            .hydroflow_ir()
        {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_snapshot!(ir.surface_syntax_string());
            });
        }
    }
}
