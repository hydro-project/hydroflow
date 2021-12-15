use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Cursor};
use std::rc::Rc;

use criterion::{criterion_group, criterion_main, Criterion};
use differential_dataflow::input::Input;
use differential_dataflow::operators::{Iterate, Join, Threshold};
use hydroflow::scheduled::graph_ext::GraphExt;

lazy_static::lazy_static! {
    static ref EDGES: HashMap<usize, Vec<usize>> = {
        let cursor = Cursor::new(include_bytes!("reachability_edges.txt"));
        let reader = BufReader::new(cursor);

        let mut edges = HashMap::new();
        for line in reader.lines() {
            let line = line.unwrap();
            let mut nums = line.split_whitespace();
            let a = nums.next().unwrap().parse().unwrap();
            let b = nums.next().unwrap().parse().unwrap();
            assert!(nums.next().is_none());
            edges.entry(a).or_insert_with(Vec::new).push(b);
        }
        edges
    };
    static ref EDGE_VEC: Vec<(usize, usize)> = {
        let cursor = Cursor::new(include_bytes!("reachability_edges.txt"));
        let reader = BufReader::new(cursor);

        reader.lines().map(|line| {
            let line = line.unwrap();
            let mut v = line.split_whitespace().map(|n| n.parse::<usize>().unwrap());
            (v.next().unwrap(), v.next().unwrap())
        }).collect()
    };
    static ref REACHABLE: HashSet<usize> = {
        let cursor = Cursor::new(include_bytes!("reachability_reachable.txt"));
        let reader = BufReader::new(cursor);

        let mut set = HashSet::new();
        for line in reader.lines() {
            let line = line.unwrap();
            set.insert(line.parse().unwrap());
        }
        set
    };
}

fn benchmark_timely(c: &mut Criterion) {
    use timely::dataflow::operators::{
        Capture, Concat, ConnectLoop, Feedback, Filter, Map, ToStream,
    };

    let edges = &*EDGES;
    let reachable = &*REACHABLE;

    c.bench_function("reachability/timely", |b| {
        b.iter(|| {
            let edges = edges.clone();
            let receiver = timely::example(|scope| {
                let mut seen = HashSet::new();

                let (handle, stream) = scope.feedback(1);

                let stream_out = (1_usize..=1)
                    .to_stream(scope)
                    .concat(&stream)
                    .flat_map(move |x| edges.get(&x).cloned().into_iter().flatten())
                    .filter(move |x| seen.insert(*x));
                stream_out.clone().connect_loop(handle);

                stream_out.capture()
            });

            let reached: HashSet<_> = receiver
                .iter()
                .filter_map(|e| match e {
                    timely::dataflow::operators::capture::event::Event::Messages(_, vec) => {
                        Some(vec)
                    }
                    _ => None,
                })
                .flatten()
                .collect();

            assert_eq!(&reached, reachable);
        });
    });
}

fn benchmark_differential(c: &mut Criterion) {
    c.bench_function("reachability/differential", |b| {
        b.iter(move || {
            timely::execute_directly(move |worker| {
                let probe = worker.dataflow::<u32, _, _>(|scope| {
                    let edges = scope.new_collection_from(EDGE_VEC.iter().cloned()).1;
                    let roots = scope.new_collection_from(vec![1]).1;

                    let reachable = roots.iterate(|reach| {
                        edges
                            .enter(&reach.scope())
                            .semijoin(reach)
                            .map(|(_src, dst)| dst)
                            .concat(reach)
                            .distinct()
                    });

                    reachable.probe()
                });

                worker.step_while(|| !probe.done());
            });
        });
    });
}

fn benchmark_hydroflow_scheduled(c: &mut Criterion) {
    use hydroflow::lang::collections::Iter;
    use hydroflow::scheduled::ctx::{RecvCtx, SendCtx};
    use hydroflow::scheduled::graph::Hydroflow;
    use hydroflow::scheduled::handoff::VecHandoff;
    use hydroflow::{tl, tt};

    let edges = &*EDGES;
    let reachable = &*REACHABLE;

    c.bench_function("reachability/hydroflow/scheduled", |b| {
        b.iter(|| {
            // A dataflow that represents graph reachability.
            let mut df = Hydroflow::new();

            let reachable_out = df.add_source(move |_ctx, send: &SendCtx<VecHandoff<usize>>| {
                send.give(Some(1));
            });

            let seen_handle = df.add_state::<RefCell<HashSet<usize>>>(Default::default());
            let (tl!(distinct_in), tl!(distinct_out)) = df
                .add_subgraph::<_, tt!(VecHandoff<usize>), tt!(VecHandoff<usize>)>(
                    move |context, tl!(recv), tl!(send)| {
                        let mut seen_state = context.state_ref(seen_handle).borrow_mut();
                        let iter = recv
                            .take_inner()
                            .into_iter()
                            .filter(|v| seen_state.insert(*v));
                        send.give(Iter(iter));
                    },
                );

            let (merge_lhs, merge_rhs, merge_out) = df.add_binary(
                |_ctx,
                 recv1: &RecvCtx<VecHandoff<_>>,
                 recv2: &RecvCtx<VecHandoff<_>>,
                 send: &SendCtx<VecHandoff<usize>>| {
                    send.give(Iter(recv1.take_inner().into_iter()));
                    send.give(Iter(recv2.take_inner().into_iter()));
                },
            );

            let (neighbors_in, neighbors_out) =
                df.add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<_>>, send| {
                    for v in recv.take_inner() {
                        if let Some(neighbors) = edges.get(&v) {
                            send.give(Iter(neighbors.iter().copied()));
                        }
                    }
                });

            let (tee_in, tee_out1, tee_out2) = df.add_binary_out(
                |_ctx,
                 recv: &RecvCtx<VecHandoff<usize>>,
                 send1: &SendCtx<VecHandoff<usize>>,
                 send2: &SendCtx<VecHandoff<usize>>| {
                    for v in recv.take_inner() {
                        send1.give(Some(v));
                        send2.give(Some(v));
                    }
                },
            );

            let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
            let reachable_inner = reachable_verts.clone();
            let sink_in = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<_>>| {
                (*reachable_inner).borrow_mut().extend(recv.take_inner());
            });

            df.add_edge(reachable_out, merge_lhs);
            df.add_edge(neighbors_out, merge_rhs);
            df.add_edge(merge_out, distinct_in);
            df.add_edge(distinct_out, tee_in);
            df.add_edge(tee_out1, neighbors_in);
            df.add_edge(tee_out2, sink_in);

            df.tick();

            assert_eq!(&*reachable_verts.borrow(), reachable);
        });
    });
}

// TODO(justin): update to use new pusherator API.
// fn benchmark_hydroflow(c: &mut Criterion) {
//     use hydroflow::compiled::{ForEach, Pivot, Tee};
//     use hydroflow::scheduled::ctx::{RecvCtx, SendCtx};
//     use hydroflow::scheduled::graph::Hydroflow;
//     use hydroflow::scheduled::handoff::VecHandoff;
//     use hydroflow::{tl, tt};

//     let edges = &*EDGES;
//     let reachable = &*REACHABLE;

//     c.bench_function("reachability/hydroflow", |b| {
//         b.iter(|| {
//             // A dataflow that represents graph reachability.
//             let mut df = Hydroflow::new();

//             let reachable_out = df.add_source(move |_ctx, send: &SendCtx<VecHandoff<usize>>| {
//                 send.give(Some(1));
//             });

//             let seen_handle = df.add_state::<RefCell<HashSet<usize>>>(Default::default());

//             type MainIn = tt!(VecHandoff<usize>, VecHandoff<usize>);
//             type MainOut = tt!(VecHandoff<usize>, VecHandoff<usize>);
//             let (tl!(origins_in, possible_reach_in), tl!(did_reach_out, output_out)) = df
//                 .add_subgraph::<_, MainIn, MainOut>(
//                     move |context, tl!(origins, did_reach_recv), tl!(did_reach_send, output)| {
//                         let origins = origins.take_inner().into_iter();
//                         let possible_reach = did_reach_recv
//                             .take_inner()
//                             .into_iter()
//                             .filter_map(|v| edges.get(&v))
//                             .flatten()
//                             .copied();

//                         let mut seen_state = context.state_ref(seen_handle).borrow_mut();
//                         let pull = origins
//                             .chain(possible_reach)
//                             .filter(|v| seen_state.insert(*v));

//                         let push_reach = ForEach::new(|v| {
//                             did_reach_send.give(Some(v));
//                         });
//                         let push_output = ForEach::new(|v| {
//                             output.give(Some(v));
//                         });
//                         let push = Tee::new(push_reach, push_output);

//                         let pivot = Pivot::new(pull, push);
//                         pivot.run();
//                     },
//                 );

//             let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
//             let reachable_inner = reachable_verts.clone();
//             let sink_in = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<_>>| {
//                 (*reachable_inner).borrow_mut().extend(recv.take_inner());
//             });

//             df.add_edge(reachable_out, origins_in);
//             df.add_edge(did_reach_out, possible_reach_in);
//             df.add_edge(output_out, sink_in);

//             df.tick();

//             assert_eq!(&*reachable_verts.borrow(), reachable);
//         });
//     });
// }

criterion_group!(
    reachability,
    benchmark_timely,
    benchmark_differential,
    benchmark_hydroflow_scheduled,
);
criterion_main!(reachability);
