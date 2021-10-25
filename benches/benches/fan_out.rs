use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hydroflow::scheduled::{collections::Iter, query::Query as Q};
use timely::dataflow::operators::{Map, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;

fn benchmark_hydroflow(c: &mut Criterion) {
    c.bench_function("fan_out/hydroflow", |b| {
        b.iter(|| {
            let mut q = Q::new();

            let source = q.source(|send| {
                send.give(Iter(0..NUM_INTS));
            });

            for op in source.tee(NUM_OPS) {
                let _ = op.sink(|x| {
                    black_box(x);
                });
            }

            q.run();
        })
    });
}

fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("fan_out/babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let source = q.source(|send| {
                send.give_iterator(0..NUM_INTS);
            });

            let _sinks: Vec<_> = (0..NUM_OPS)
                .map(|_| source.clone().map(black_box))
                .collect();

            (*q.df).borrow_mut().run();
        })
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("fan_out/timely", |b| {
        b.iter(|| {
            timely::example(move |scope| {
                let source = (0..NUM_INTS).to_stream(scope);

                let _sinks: Vec<_> = (0..NUM_OPS)
                    .map(|_| source.clone().map(black_box))
                    .collect();
            });
        })
    });
}

fn benchmark_spinachflow_asym(c: &mut Criterion) {
    c.bench_function("fan_out/spinachflow (asym)", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| {
            async {
                use spinachflow::futures::StreamExt;

                let local_set = spinachflow::tokio::task::LocalSet::new();

                let stream = spinachflow::futures::stream::iter(0..NUM_INTS);
                let mut asym_split = spinachflow::stream::AsymSplit::new(stream);

                // N - 1
                for _ in 1..NUM_OPS {
                    let split = asym_split.add_split();
                    local_set.spawn_local(async move {
                        let mut split = split;
                        loop {
                            let item = split.next().await;
                            if item.is_none() {
                                break;
                            }
                        }
                    });
                }
                // 1
                local_set.spawn_local(async move {
                    let mut split = asym_split;
                    loop {
                        let item = split.next().await;
                        if item.is_none() {
                            break;
                        }
                    }
                });

                local_set.await;
            }
        });
    });
}

fn benchmark_sol(c: &mut Criterion) {
    c.bench_function("fan_out/sol", |b| {
        b.iter(|| {
            for x in 0..NUM_INTS {
                for _ in 1..NUM_OPS {
                    black_box(x);
                }
            }
        })
    });
}

criterion_group!(
    fan_out_dataflow,
    benchmark_hydroflow,
    benchmark_babyflow,
    benchmark_timely,
    benchmark_spinachflow_asym,
    benchmark_sol,
);
criterion_main!(fan_out_dataflow);
