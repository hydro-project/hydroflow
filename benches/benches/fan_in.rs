use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::query::Query as Q;
use timely::dataflow::operators::{Concatenate, Inspect, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;

fn make_ints(i: usize) -> impl Iterator<Item = usize> {
    (i * NUM_INTS)..((i + 1) * NUM_INTS)
}

fn benchmark_hydroflow(c: &mut Criterion) {
    c.bench_function("fan_in/hydroflow", |b| {
        b.iter(|| {
            let mut q = Q::new();

            let sources: Vec<_> = (0..NUM_OPS)
                .map(|i| {
                    q.source(move |_ctx, send| {
                        send.give(Iter(make_ints(i)));
                    })
                })
                .collect();

            let op = q.concat(sources);

            op.sink(move |v| {
                black_box(v);
            });

            q.run_available();
        })
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("fan_in/timely", |b| {
        b.iter(|| {
            timely::example(move |scope| {
                let sources: Vec<_> = (0..NUM_OPS)
                    .map(|i| make_ints(i).to_stream(scope))
                    .collect();

                let merged = scope.concatenate(sources);

                merged.inspect(|x| {
                    black_box(x);
                });
            });
        })
    });
}

fn benchmark_iters(c: &mut Criterion) {
    c.bench_function("fan_in/iters", |b| {
        b.iter(|| {
            (0..NUM_OPS).flat_map(make_ints).for_each(|x| {
                black_box(x);
            });
        });
    });
}

fn benchmark_for_loops(c: &mut Criterion) {
    c.bench_function("fan_in/loops", |b| {
        b.iter(|| {
            let iters: Vec<_> = (0..NUM_OPS).map(make_ints).collect();
            for iter in iters {
                for x in iter {
                    black_box(x);
                }
            }
        });
    });
}

criterion_group!(
    fan_in_dataflow,
    benchmark_hydroflow,
    benchmark_timely,
    benchmark_iters,
    benchmark_for_loops,
);

// use pprof::criterion::{Output, PProfProfiler};
// criterion_group!(
//     name = fan_in_dataflow;
//     config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
//     targets =
//         benchmark_babyflow,
//         benchmark_hydroflow,
//         benchmark_timely,
//         benchmark_spinachflow,
//         benchmark_iters,
//         benchmark_for_loops,
// );
criterion_main!(fan_in_dataflow);
