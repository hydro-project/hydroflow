use std::marker::PhantomData;

use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use timely::dataflow::operators::{Inspect, Map, ToStream};

const NUM_OPS: usize = 20;
const NUM_ROWS: usize = 100_000;
const STARTING_STRING: &str = "foobar";

trait Operation {
    fn name() -> &'static str;
    fn action(s: String) -> String;
}

struct UpcaseInPlace;
impl Operation for UpcaseInPlace {
    fn name() -> &'static str {
        "upcase_in_place"
    }

    fn action(mut s: String) -> String {
        s.make_ascii_uppercase();
        s
    }
}

struct UpcaseAllocating;
impl Operation for UpcaseAllocating {
    fn name() -> &'static str {
        "upcase_allocating"
    }

    fn action(s: String) -> String {
        s.to_uppercase()
    }
}

struct Concatting;
impl Operation for Concatting {
    fn name() -> &'static str {
        "concatting"
    }

    fn action(mut s: String) -> String {
        s.push_str("barfoo");
        s
    }
}

// This benchmark runs babyflow which more-or-less just copies the data directly
// between the operators, but with some extra overhead.
fn benchmark_babyflow<O: 'static + Operation>(c: &mut Criterion) {
    c.bench_function(format!("{}/babyflow", O::name()).as_str(), |b| {
        b.iter(|| {
            let mut q = Query::new();

            let mut op = q.source(move |send| {
                for _ in 0..NUM_ROWS {
                    send.push(STARTING_STRING.to_owned());
                }
            });

            for _ in 0..NUM_OPS {
                op = op.map(O::action);
            }

            op.sink(|i| {
                black_box(i);
            });

            (*q.df).borrow_mut().run();
        })
    });
}

// This benchmark just copies around a bunch of data with basically zero
// overhead, so this should theoretically be the fastest achievable (with a
// single thread).
fn benchmark_raw_copy<O: 'static + Operation>(c: &mut Criterion) {
    c.bench_function(format!("{}/raw", O::name()).as_str(), |b| {
        b.iter(|| {
            let mut data: Vec<_> = (0..NUM_ROWS).map(|_| STARTING_STRING.to_owned()).collect();
            let mut next = Vec::new();

            for _ in 0..NUM_OPS {
                next.extend(data.drain(..).map(O::action));
                std::mem::swap(&mut data, &mut next);
            }

            for elt in data {
                black_box(elt);
            }
        })
    });
}

fn benchmark_iter<O: 'static + Operation>(c: &mut Criterion) {
    c.bench_function(format!("{}/iter", O::name()).as_str(), |b| {
        b.iter(|| {
            let iter = (0..NUM_ROWS).map(|_| STARTING_STRING.to_owned());

            ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
            seq_macro::seq!(_ in 0..20 {
                let iter = iter.map(O::action);
            });

            let data: Vec<_> = iter.collect();

            for elt in data {
                black_box(elt);
            }
        });
    });
}

async fn benchmark_spinach<O: 'static + Operation>(num_ints: usize) {
    type MyLatRepr = spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, String>;
    let op = <spinachflow::op::OnceOp<MyLatRepr>>::new(
        (0..num_ints).map(|_| STARTING_STRING.to_owned()).collect(),
    );

    struct MyMorphism<O: 'static + Operation>(PhantomData<O>);
    impl<O: 'static + Operation> spinachflow::func::unary::Morphism for MyMorphism<O> {
        type InLatRepr = MyLatRepr;
        type OutLatRepr = MyLatRepr;
        fn call<Y: spinachflow::hide::Qualifier>(
            &self,
            item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
        ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
            item.map(O::action)
        }
    }

    ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
    seq_macro::seq!(N in 0..20 {
        let op = spinachflow::op::MorphismOp::new(op, MyMorphism::<O>(PhantomData));
    });

    let comp = spinachflow::comp::NullComp::new(op);
    spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
}

fn criterion_spinach<O: 'static + Operation>(c: &mut Criterion) {
    c.bench_function(format!("{}/spinach", O::name()).as_str(), |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| benchmark_spinach::<O>(NUM_ROWS));
    });
}

fn benchmark_timely<O: 'static + Operation>(c: &mut Criterion) {
    c.bench_function(format!("{}/timely", O::name()).as_str(), |b| {
        b.iter(|| {
            timely::example(|scope| {
                let mut op = (0..NUM_ROWS)
                    .map(|_| STARTING_STRING.to_owned())
                    .to_stream(scope);

                for _ in 0..NUM_OPS {
                    op = op.map(O::action);
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
    benchmark_babyflow::<UpcaseInPlace>,
    benchmark_babyflow::<UpcaseAllocating>,
    benchmark_babyflow::<Concatting>,
    benchmark_timely::<UpcaseInPlace>,
    benchmark_timely::<UpcaseAllocating>,
    benchmark_timely::<Concatting>,
    benchmark_raw_copy::<UpcaseInPlace>,
    benchmark_raw_copy::<UpcaseAllocating>,
    benchmark_raw_copy::<Concatting>,
    benchmark_iter::<UpcaseInPlace>,
    benchmark_iter::<UpcaseAllocating>,
    benchmark_iter::<Concatting>,
    criterion_spinach::<UpcaseInPlace>,
    criterion_spinach::<UpcaseAllocating>,
    criterion_spinach::<Concatting>,
);
criterion_main!(upcase_dataflow);
