use hydroflow_plus::*;

pub struct Leader {}
pub struct Worker {}

pub fn map_reduce<'a>(flow: &FlowBuilder<'a>) -> (Process<'a, Leader>, Cluster<'a, Worker>) {
    let process = flow.process();
    let cluster = flow.cluster();

    let words = process
        .source_iter(q!(vec!["abc", "abc", "xyz", "abc"]))
        .map(q!(|s| s.to_string()));

    let partitioned_words = words
        .round_robin_bincode(&cluster)
        .map(q!(|string| (string, ())));

    let batches = unsafe {
        // SAFETY: addition is associative so we can batch reduce
        partitioned_words.tick_batch(&cluster.tick())
    }
    .fold_keyed(q!(|| 0), q!(|count, _| *count += 1))
    .inspect(q!(|(string, count)| println!(
        "partition count: {} - {}",
        string, count
    )))
    .all_ticks()
    .send_bincode_interleaved(&process);

    unsafe {
        // SAFETY: addition is associative so we can batch reduce
        batches
            .tick_prefix(&process.tick())
            .reduce_keyed_commutative(q!(|total, count| *total += count))
    }
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
        let built = builder.with_default_optimize::<DeployRuntime>();

        insta::assert_debug_snapshot!(built.ir());

        for (id, ir) in built.compile(&RuntimeData::new("FAKE")).hydroflow_ir() {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_snapshot!(ir.surface_syntax_string());
            });
        }
    }
}
