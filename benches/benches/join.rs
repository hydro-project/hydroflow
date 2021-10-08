use std::collections::HashMap;

use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use timely::dataflow::{
    channels::pact::Pipeline,
    operators::{Operator, ToStream},
};

// const NUM_OPS: usize = 20;
const NUM_INTS: usize = 100_000;

fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let lhs = q.source(move |send| {
                send.give_iterator((0..NUM_INTS).map(|x| (x, x)));
            });
            let rhs = q.source(move |send| {
                send.give_iterator((0..NUM_INTS).map(|x| (x, x)));
            });

            let op = lhs.join(rhs);

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
                let lhs = (0..NUM_INTS).map(|x| (x, x)).to_stream(scope);
                let rhs = (0..NUM_INTS).map(|x| (x, x)).to_stream(scope);

                lhs.binary(&rhs, Pipeline, Pipeline, "HashJoin", |_, _| {
                    let mut left_tab: HashMap<usize, Vec<usize>> = HashMap::new();
                    let mut right_tab: HashMap<usize, Vec<usize>> = HashMap::new();
                    let mut lvec: Vec<(usize, usize)> = Vec::new();
                    let mut rvec: Vec<(usize, usize)> = Vec::new();
                    move |left, right, output| {
                        left.for_each(|time, data| {
                            data.swap(&mut lvec);
                            let mut session = output.session(&time);

                            for (k, v) in lvec.drain(..) {
                                if let Some(matches) = right_tab.get(&k) {
                                    for v2 in matches {
                                        session.give((k, v, v2.clone()))
                                    }
                                }

                                left_tab.entry(k).or_insert_with(Vec::new).push(v);
                            }
                        });

                        right.for_each(|time, data| {
                            data.swap(&mut rvec);
                            let mut session = output.session(&time);

                            for (k, v) in rvec.drain(..) {
                                if let Some(matches) = left_tab.get(&k) {
                                    for v2 in matches {
                                        session.give((k, v2.clone(), v))
                                    }
                                }

                                right_tab.entry(k).or_insert_with(Vec::new).push(v);
                            }
                        });
                    }
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
                use spinachflow::futures::future::ready;

                let stream_a = spinachflow::futures::stream::iter((0..NUM_INTS).map(|x| (x, x)));
                let stream_b = spinachflow::futures::stream::iter((0..NUM_INTS).map(|x| (x, x)));
                let stream = spinachflow::stream::Join::new(stream_a, stream_b);

                let stream = stream.map(|x| ready(black_box(x)));
                let mut stream = stream;
                while stream.next().await.is_some() {}
            }
        });
    });
}

fn benchmark_sol(c: &mut Criterion) {
    c.bench_function("sol", |b| {
        b.iter(|| {
            let iter_a = (0..NUM_INTS).map(|x| (x, x));
            let iter_b = (0..NUM_INTS).map(|x| (x, x));
            let mut items_a = HashMap::new();
            let mut items_b = HashMap::new();

            for (key, val_a) in iter_a {
                items_a.entry(key)
                    .or_insert_with(Vec::new)
                    .push(val_a);
                if let Some(vals_b) = items_b.get(&key) {
                    for val_b in vals_b {
                        black_box((key, val_a, val_b));
                    }
                }
            }
            for (key, val_b) in iter_b {
                items_b.entry(key)
                    .or_insert_with(Vec::new)
                    .push(val_b);
                if let Some(vals_a) = items_a.get(&key) {
                    for val_a in vals_a {
                        black_box((key, val_a, val_b));
                    }
                }
            }
        });
    });
}

criterion_group!(fan_in_dataflow,
    benchmark_babyflow,
    benchmark_timely,
    benchmark_spinachflow,
    benchmark_sol,
);
criterion_main!(fan_in_dataflow);
