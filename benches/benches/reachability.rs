use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Cursor};
use std::rc::Rc;

use criterion::{criterion_group, criterion_main, Criterion};

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

fn benchmark_hydroflow(c: &mut Criterion) {
    use hydroflow::scheduled::collections::Iter;
    use hydroflow::scheduled::handoff::VecHandoff;
    use hydroflow::scheduled::{Hydroflow, RecvCtx, SendCtx};

    let edges = &*EDGES;
    let reachable = &*REACHABLE;

    c.bench_function("reachability/hydroflow", |b| {
        b.iter(|| {
            // A dataflow that represents graph reachability.
            let mut df = Hydroflow::new();

            let reachable_out = df.add_source(move |send: &mut SendCtx<VecHandoff<usize>>| {
                send.give(Some(1));
            });

            let mut seen = HashSet::new();
            let (distinct_in, distinct_out) = df.add_inout(
                move |recv: &mut RecvCtx<VecHandoff<usize>>,
                      send: &mut SendCtx<VecHandoff<usize>>| {
                    let iter = recv.into_iter().filter(|v| seen.insert(*v));
                    send.give(Iter(iter));
                },
            );

            let (merge_lhs, merge_rhs, merge_out) =
                df.add_binary(|recv1, recv2, send: &mut SendCtx<VecHandoff<usize>>| {
                    send.give(Iter(recv1.into_iter()));
                    send.give(Iter(recv2.into_iter()));
                });

            let (neighbors_in, neighbors_out) = df.add_inout(move |recv, send| {
                for v in &*recv {
                    if let Some(neighbors) = edges.get(&v) {
                        send.give(Iter(neighbors.iter().copied()));
                    }
                }
            });

            let (tee_in, tee_out1, tee_out2) = df.add_binary_out(
                |recv: &mut RecvCtx<VecHandoff<usize>>,
                 send1: &mut SendCtx<VecHandoff<usize>>,
                 send2: &mut SendCtx<VecHandoff<usize>>| {
                    for v in &*recv {
                        send1.give(Some(v));
                        send2.give(Some(v));
                    }
                },
            );

            let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
            let reachable_inner = reachable_verts.clone();
            let sink_in = df.add_sink(move |recv| {
                (*reachable_inner).borrow_mut().extend(recv.into_iter());
            });

            df.add_edge(reachable_out, merge_lhs);
            df.add_edge(neighbors_out, merge_rhs);
            df.add_edge(merge_out, distinct_in);
            df.add_edge(distinct_out, tee_in);
            df.add_edge(tee_out1, neighbors_in);
            df.add_edge(tee_out2, sink_in);

            df.run();

            assert_eq!(&*reachable_verts.borrow(), reachable);
        });
    });
}

criterion_group!(
    reachability,
    benchmark_timely,
    // benchmark_babyflow,
    benchmark_hydroflow,
);
criterion_main!(reachability);
