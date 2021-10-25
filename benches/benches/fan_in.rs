use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hydroflow::scheduled::{collections::Iter, query::Query as Q};
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
                    q.source(move |send| {
                        send.give(Iter(make_ints(i)));
                    })
                })
                .collect();

            let op = q.concat(sources);

            op.sink(move |v| {
                black_box(v);
            });

            q.run();
        })
    });
}

fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("fan_in/babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let sources: Vec<_> = (0..NUM_OPS)
                .map(|i| {
                    q.source(move |send| {
                        send.give_iterator(make_ints(i));
                    })
                })
                .collect();

            let op = q.concat(sources);

            op.sink(move |v| {
                black_box(v);
            });

            (*q.df).borrow_mut().run();
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

fn benchmark_spinachflow(c: &mut Criterion) {
    c.bench_function("fan_in/spinachflow", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| async {
            use spinachflow::futures::future::ready;
            use spinachflow::futures::StreamExt;

            let mut i = 0;
            let streams = [(); NUM_OPS].map(|_| {
                i += 1;
                spinachflow::futures::stream::iter(make_ints(i - 1))
            });
            let stream = spinachflow::stream::SelectArr::new(streams);
            let stream = stream.map(|x| ready(black_box(x)));
            let mut stream = stream;
            while stream.next().await.is_some() {}
        });
    });
}

fn benchmark_iters(c: &mut Criterion) {
    c.bench_function("fan_in/iters", |b| {
        b.iter(|| {
            (0..NUM_OPS).map(make_ints).flatten().for_each(|x| {
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
    benchmark_babyflow,
    benchmark_hydroflow,
    benchmark_timely,
    benchmark_spinachflow,
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
