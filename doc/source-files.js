var sourcesIndex = JSON.parse('{\
"hydro":["",[["core",[["hydroflow_crate",[],["build.rs","mod.rs","ports.rs"]]],["custom_service.rs","deployment.rs","gcp.rs","localhost.rs","mod.rs","progress.rs","ssh.rs","terraform.rs","util.rs"]]],["cli.rs","lib.rs"]],\
"hydroflow":["",[["compiled",[["pull",[["half_join_state",[],["mod.rs","multiset.rs","set.rs"]]],["cross_join.rs","mod.rs","symmetric_hash_join.rs","symmetric_hash_join_lattice.rs"]]],["mod.rs","push_handoff.rs"]],["lang",[],["clear.rs","mod.rs","monotonic_map.rs"]],["props",[],["mod.rs","wrap.rs"]],["scheduled",[["handoff",[],["handoff_list.rs","mod.rs","tee.rs","vector.rs"]],["net",[],["mod.rs","network_vertex.rs"]]],["context.rs","graph.rs","graph_ext.rs","input.rs","mod.rs","port.rs","query.rs","reactor.rs","state.rs","subgraph.rs"]],["util",[],["mod.rs","socket.rs","tcp.rs","udp.rs"]]],["declarative_macro.rs","lib.rs"]],\
"hydroflow_cli_integration":["",[],["lib.rs"]],\
"hydroflow_datalog":["",[],["lib.rs"]],\
"hydroflow_datalog_core":["",[],["grammar.rs","join_plan.rs","lib.rs","util.rs"]],\
"hydroflow_internalmacro":["",[],["lib.rs"]],\
"hydroflow_lang":["",[["graph",[["ops",[],["anti_join.rs","batch.rs","cross_join.rs","demux.rs","dest_file.rs","dest_sink.rs","dest_sink_serde.rs","difference.rs","enumerate.rs","filter.rs","filter_map.rs","flat_map.rs","flatten.rs","fold.rs","for_each.rs","group_by.rs","identity.rs","initialize.rs","inspect.rs","join.rs","keyed_fold.rs","keyed_reduce.rs","lattice_join.rs","lattice_merge.rs","map.rs","merge.rs","mod.rs","next_stratum.rs","next_tick.rs","null.rs","persist.rs","reduce.rs","repeat_fn.rs","repeat_iter.rs","repeat_iter_external.rs","sort.rs","sort_by.rs","source_file.rs","source_interval.rs","source_iter.rs","source_json.rs","source_stdin.rs","source_stream.rs","source_stream_serde.rs","tee.rs","unique.rs","unzip.rs"]]],["di_mul_graph.rs","eliminate_extra_merges_tees.rs","flat_graph_builder.rs","flat_to_partitioned.rs","graph_algorithms.rs","graph_write.rs","hydroflow_graph.rs","mod.rs"]]],["diagnostic.rs","lib.rs","parse.rs","pretty_span.rs","union_find.rs"]],\
"hydroflow_macro":["",[],["lib.rs"]],\
"lattices":["",[],["bottom.rs","collections.rs","dom_pair.rs","fake.rs","lib.rs","map_union.rs","ord.rs","pair.rs","set_union.rs"]],\
"multiplatform_test":["",[],["lib.rs"]],\
"pusherator":["",[],["demux.rs","filter.rs","filter_map.rs","flatten.rs","for_each.rs","inspect.rs","lib.rs","map.rs","partition.rs","pivot.rs","switch.rs","tee.rs","unzip.rs"]],\
"relalg":["",[],["codegen.rs","lib.rs","runtime.rs","sexp.rs"]],\
"variadics":["",[],["lib.rs"]]\
}');
createSourceSidebar();
