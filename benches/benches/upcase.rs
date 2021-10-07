#![allow(dead_code, unused_imports)]

use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc::channel;
use std::thread::{self, sleep};
use std::time::Duration;
use timely::dataflow::operators::{Inspect, Map, ToStream};

const NUM_OPS: usize = 20;
const NUM_ROWS: usize = 1_000_000;
const STARTING_STRING: &str = "foobar";

fn operation(mut s: String) -> String {
    s.make_ascii_uppercase();
    s
}

// fn operation(s: String) -> String {
//     s
// }

// This benchmark runs babyflow which more-or-less just copies the data directly
// between the operators, but with some extra overhead.
fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let mut op = q.source(move |send| {
                for _ in 0..NUM_ROWS {
                    send.push(STARTING_STRING.to_owned());
                }
            });

            for _ in 0..NUM_OPS {
                op = op.map(operation);
            }

            op.sink(|i| {
                black_box(i);
            });

            (*q.df).borrow_mut().run();
        })
    });
}

fn benchmark_pipeline(c: &mut Criterion) {
    c.bench_function("pipeline", |b| {
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

            for i in 0..NUM_ROWS {
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
    c.bench_function("raw copy", |b| {
        b.iter(|| {
            let mut data: Vec<_> = (0..NUM_ROWS).map(|_| STARTING_STRING.to_owned()).collect();
            let mut next = Vec::new();

            for _ in 0..NUM_OPS {
                next.extend(data.drain(..).map(operation));
                std::mem::swap(&mut data, &mut next);
            }

            for elt in data {
                black_box(elt);
            }
        })
    });
}

fn benchmark_iter(c: &mut Criterion) {
    c.bench_function("iter", |b| {
        b.iter(|| {
            let data: Vec<_> = (0..NUM_ROWS).map(|_| STARTING_STRING.to_owned()).collect();

            let iter = data.into_iter();

            ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
            seq_macro::seq!(_ in 0..20 {
                let iter = iter.map(operation);
            });

            let data: Vec<_> = iter.collect();

            for elt in data {
                black_box(elt);
            }
        });
    });
}

async fn benchmark_spinach(num_ints: usize) {
    use spinachflow::comp::Comp;

    type MyLatRepr = spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, String>;
    let op = <spinachflow::op::OnceOp<MyLatRepr>>::new(
        (0..num_ints).map(|_| STARTING_STRING.to_owned()).collect(),
    );

    struct MyMorphism();
    impl spinachflow::func::unary::Morphism for MyMorphism {
        type InLatRepr = MyLatRepr;
        type OutLatRepr = MyLatRepr;
        fn call<Y: spinachflow::hide::Qualifier>(
            &self,
            item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
        ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
            item.map(operation)
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
    c.bench_function("spinach", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| benchmark_spinach(NUM_ROWS));
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("timely", |b| {
        b.iter(|| {
            timely::example(|scope| {
                let mut op = (0..NUM_ROWS)
                    .map(|_| STARTING_STRING.to_owned())
                    .to_stream(scope);

                for _ in 0..NUM_OPS {
                    op = op.map(operation);
                }

                op.inspect(|i| {
                    black_box(i);
                });
            });
        })
    });
}

criterion_group!(
    upcase_dataflow,
    benchmark_timely,
    benchmark_babyflow,
    benchmark_raw_copy,
    benchmark_iter,
    criterion_spinach,
);
criterion_main!(upcase_dataflow);
