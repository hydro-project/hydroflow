use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use timely::dataflow::operators::{Concatenate, Inspect, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;

fn make_ints(i: usize) -> impl Iterator<Item = usize> {
    (i * NUM_INTS)..((i + 1) * NUM_INTS)
}

fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("babyflow", |b| {
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
    c.bench_function("timely", |b| {
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
    c.bench_function("spinachflow", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| {
            async {
                use spinachflow::futures::StreamExt;

                let mut i = 0;
                let streams = [(); NUM_OPS].map(|_| {
                    i += 1;
                    spinachflow::futures::stream::iter(make_ints(i - 1))
                });
                let stream = spinachflow::stream::SelectArr::new(streams);
                let mut stream = stream;
                loop {
                    let item = stream.next().await;
                    if item.map(black_box).is_none() {
                        break;
                    }
                }
            }
        });
    });
}

criterion_group!(fan_in_dataflow,
    benchmark_babyflow,
    benchmark_timely,
    benchmark_spinachflow);
criterion_main!(fan_in_dataflow);
