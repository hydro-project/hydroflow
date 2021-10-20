use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc::channel;
use std::thread;
use timely::dataflow::operators::{Inspect, Map, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;

// This benchmark runs babyflow which more-or-less just copies the data directly
// between the operators, but with some extra overhead.
fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("identity/babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let mut op = q.source(move |send| {
                for i in 0..NUM_INTS {
                    send.push(i);
                }
            });

            for _ in 0..NUM_OPS {
                op = op.map(black_box);
            }

            op.sink(|i| {
                black_box(i);
            });

            (*q.df).borrow_mut().run();
        })
    });
}

fn benchmark_pipeline(c: &mut Criterion) {
    c.bench_function("identity/pipeline", |b| {
        b.iter(|| {
            let (input, mut output) = channel();

            for _ in 0..NUM_OPS {
                let (tx, mut rx) = channel();
                std::mem::swap(&mut output, &mut rx);
                thread::spawn(move || {
                    for elt in rx {
                        tx.send(elt).unwrap();
                    }
                });
            }

            for i in 0..NUM_INTS {
                input.send(i).unwrap();
            }
            drop(input);
            for elt in output {
                black_box(elt);
            }
        });
    });
}

// This benchmark just copies around a bunch of data with basically zero
// overhead, so this should theoretically be the fastest achievable (with a
// single thread).
fn benchmark_raw_copy(c: &mut Criterion) {
    c.bench_function("identity/raw", |b| {
        b.iter(|| {
            let mut data: Vec<_> = (0..NUM_INTS).collect();
            let mut next = Vec::new();

            for _ in 0..NUM_OPS {
                next.append(&mut data);
                std::mem::swap(&mut data, &mut next);
            }

            for elt in data {
                black_box(elt);
            }
        })
    });
}

fn benchmark_iter(c: &mut Criterion) {
    c.bench_function("identity/iter", |b| {
        b.iter(|| {
            let data: Vec<_> = (0..NUM_INTS).collect();

            let iter = data.into_iter();

            ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
            seq_macro::seq!(_ in 0..20 {
                let iter = iter.map(black_box);
            });

            let data: Vec<_> = iter.collect();

            for elt in data {
                black_box(elt);
            }
        });
    });
}

fn benchmark_iter_collect(c: &mut Criterion) {
    c.bench_function("identity/iter-collect", |b| {
        b.iter(|| {
            let mut data: Vec<_> = (0..NUM_INTS).collect();

            for _ in 0..NUM_OPS {
                let iter = data.into_iter();
                let iter = iter.map(black_box);
                data = iter.collect();
            }

            for elt in data {
                black_box(elt);
            }
        });
    });
}

async fn benchmark_spinach(num_ints: usize) {
    type MyLatRepr = spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, usize>;
    let op = <spinachflow::op::OnceOp<MyLatRepr>>::new((0..num_ints).collect());

    struct MyMorphism();
    impl spinachflow::func::unary::Morphism for MyMorphism {
        type InLatRepr = MyLatRepr;
        type OutLatRepr = MyLatRepr;
        fn call<Y: spinachflow::hide::Qualifier>(
            &self,
            item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
        ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
            item.map(black_box)
        }
    }

    ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
    seq_macro::seq!(N in 0..20 {
        let op = spinachflow::op::MorphismOp::new(op, MyMorphism());
    });

    let comp = spinachflow::comp::NullComp::new(op);
    spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
}

fn criterion_spinach(c: &mut Criterion) {
    c.bench_function("identity/spinach", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| benchmark_spinach(NUM_INTS));
    });
}

fn benchmark_spinach_chunks(num_ints: usize) -> impl std::future::Future {
    type MyLatRepr = spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, usize>;

    struct MyMorphism();
    impl spinachflow::func::unary::Morphism for MyMorphism {
        type InLatRepr = MyLatRepr;
        type OutLatRepr = MyLatRepr;
        fn call<Y: spinachflow::hide::Qualifier>(
            &self,
            item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
        ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
            item.map(black_box)
        }
    }

    let data: Vec<_> = (0..num_ints).collect();
    let chunks: Vec<Vec<Vec<_>>> = data
        .chunks(100 * 100)
        .map(|chunk| chunk.iter().copied().collect())
        .map(|chunk_vec: Vec<_>| {
            chunk_vec
                .chunks(100)
                .map(|chunk| chunk.iter().copied().collect())
                .collect()
        })
        .collect();

    let local = tokio::task::LocalSet::new();

    for chunk in chunks {
        let op = <spinachflow::op::IterOp<MyLatRepr, _>>::new(chunk);

        ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
        seq_macro::seq!(N in 0..20 {
            let op = spinachflow::op::MorphismOp::new(op, MyMorphism());
        });

        let comp = spinachflow::comp::NullComp::new(op);
        local.spawn_local(async move {
            spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
        });
    }
    local
}

fn criterion_spinach_chunks(c: &mut Criterion) {
    c.bench_function("identity/spinach (size 10_000 chunks in 100 tasks)", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| benchmark_spinach_chunks(NUM_INTS));
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("identity/timely", |b| {
        b.iter(|| {
            timely::example(|scope| {
                let mut op = (0..NUM_INTS).to_stream(scope);
                for _ in 0..NUM_OPS {
                    op = op.map(black_box)
                }

                op.inspect(|i| {
                    black_box(i);
                });
            });
        })
    });
}

criterion_group!(
    identity_dataflow,
    benchmark_timely,
    benchmark_babyflow,
    criterion_spinach,
    criterion_spinach_chunks,
    benchmark_pipeline,
    benchmark_iter,
    benchmark_iter_collect,
    benchmark_raw_copy,
);
criterion_main!(identity_dataflow);
