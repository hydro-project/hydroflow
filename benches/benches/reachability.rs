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
    use hydroflow::scheduled::graph::Hydroflow;
    use hydroflow::scheduled::handoff::VecHandoff;
    use hydroflow::tl;

    let edges = &*EDGES;
    let reachable = &*REACHABLE;

    c.bench_function("reachability/hydroflow/scheduled", |b| {
        b.iter(|| {
            // A dataflow that represents graph reachability.
            let mut df = Hydroflow::new();

            type Hoff = VecHandoff<usize>;
            let (reachable_out, merge_lhs) = df.make_edge::<Hoff>();
            let (neighbors_out, merge_rhs) = df.make_edge::<Hoff>();
            let (merge_out, distinct_in) = df.make_edge::<Hoff>();
            let (distinct_out, tee_in) = df.make_edge::<Hoff>();
            let (tee_out1, neighbors_in) = df.make_edge::<Hoff>();
            let (tee_out2, sink_in) = df.make_edge::<Hoff>();

            df.add_subgraph_source(reachable_out, move |_ctx, send| {
                send.give(Some(1));
            });

            let seen_handle = df.add_state::<RefCell<HashSet<usize>>>(Default::default());
            df.add_subgraph(
                tl!(distinct_in),
                tl!(distinct_out),
                move |context, tl!(recv), tl!(send)| {
                    let mut seen_state = context.state_ref(seen_handle).borrow_mut();
                    let iter = recv
                        .take_inner()
                        .into_iter()
                        .filter(|v| seen_state.insert(*v));
                    send.give(Iter(iter));
                },
            );

            df.add_subgraph_2in_out(
                merge_lhs,
                merge_rhs,
                merge_out,
                |_ctx, recv1, recv2, send| {
                    send.give(Iter(recv1.take_inner().into_iter()));
                    send.give(Iter(recv2.take_inner().into_iter()));
                },
            );

            df.add_subgraph_in_out(neighbors_in, neighbors_out, move |_ctx, recv, send| {
                for v in recv.take_inner() {
                    if let Some(neighbors) = edges.get(&v) {
                        send.give(Iter(neighbors.iter().copied()));
                    }
                }
            });

            df.add_subgraph_in_2out(tee_in, tee_out1, tee_out2, |_ctx, recv, send1, send2| {
                for v in recv.take_inner() {
                    send1.give(Some(v));
                    send2.give(Some(v));
                }
            });

            let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
            let reachable_inner = reachable_verts.clone();
            df.add_subgraph_sink(sink_in, move |_ctx, recv| {
                (*reachable_inner).borrow_mut().extend(recv.take_inner());
            });

            df.tick();

            assert_eq!(&*reachable_verts.borrow(), reachable);
        });
    });
}

fn benchmark_hydroflow(c: &mut Criterion) {
    use hydroflow::compiled::{for_each::ForEach, IteratorToPusherator, PusheratorBuild};
    use hydroflow::scheduled::graph::Hydroflow;
    use hydroflow::scheduled::handoff::VecHandoff;
    use hydroflow::tl;

    let edges = &*EDGES;
    let reachable = &*REACHABLE;

    c.bench_function("reachability/hydroflow", |b| {
        b.iter(|| {
            // A dataflow that represents graph reachability.
            let mut df = Hydroflow::new();

            let (reachable_out, origins_in) = df.make_edge::<VecHandoff<usize>>();
            let (did_reach_out, possible_reach_in) = df.make_edge::<VecHandoff<usize>>();
            let (output_out, sink_in) = df.make_edge::<VecHandoff<usize>>();

            df.add_subgraph_source(reachable_out, move |_ctx, send| {
                send.give(Some(1));
            });

            let seen_handle = df.add_state::<RefCell<HashSet<usize>>>(Default::default());

            df.add_subgraph(
                tl!(origins_in, possible_reach_in),
                tl!(did_reach_out, output_out),
                move |context, tl!(origins, did_reach_recv), tl!(did_reach_send, output)| {
                    let origins = origins.take_inner().into_iter();
                    let possible_reach = did_reach_recv
                        .take_inner()
                        .into_iter()
                        .filter_map(|v| edges.get(&v))
                        .flatten()
                        .copied();

                    let mut seen_state = context.state_ref(seen_handle).borrow_mut();
                    let pull = origins
                        .chain(possible_reach)
                        .filter(|v| seen_state.insert(*v));

                    let pivot = pull
                        .pull_to_push()
                        .tee(ForEach::new(|v| {
                            did_reach_send.give(Some(v));
                        }))
                        .for_each(|v| {
                            output.give(Some(v));
                        });

                    pivot.run();
                },
            );

            let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
            let reachable_inner = reachable_verts.clone();
            df.add_subgraph_sink(sink_in, move |_ctx, recv| {
                (*reachable_inner).borrow_mut().extend(recv.take_inner());
            });

            df.tick();

            assert_eq!(&*reachable_verts.borrow(), reachable);
        });
    });
}

criterion_group!(
    reachability,
    benchmark_timely,
    benchmark_differential,
    benchmark_hydroflow_scheduled,
    benchmark_hydroflow,
);
criterion_main!(reachability);
