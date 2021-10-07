use babyflow::babyflow::Query;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use timely::dataflow::operators::{Concatenate, Filter, Inspect, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;
const BRANCH_FACTOR: usize = 2;

fn benchmark_raw(c: &mut Criterion) {
    c.bench_function("raw", |b| {
        b.iter(|| {
            let mut evens = Vec::new();
            let mut odds = Vec::new();

            let mut data: Vec<_> = (0..NUM_INTS).collect();

            for _ in 0..NUM_OPS {
                for i in data.drain(..) {
                    if i % 2 == 0 {
                        evens.push(i);
                    } else {
                        odds.push(i)
                    }
                }

                data.extend(evens.drain(..));
                data.extend(odds.drain(..));
            }
        })
    });
}

fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let mut op = q.source(move |send| {
                send.give_iterator(0..NUM_INTS);
            });

            for _ in 0..NUM_OPS {
                op = q.concat(
                    (0..BRANCH_FACTOR).map(|i| op.clone().filter(move |x| x % BRANCH_FACTOR == i)),
                );
            }

            op.sink(|i| {
                black_box(i);
            });

            (*q.df).borrow_mut().run();
        })
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("timely", |b| {
        b.iter(|| {
            timely::example(|scope| {
                let mut op = (0..NUM_INTS).to_stream(scope);
                for _ in 0..NUM_OPS {
                    let mut ops = Vec::new();

                    for i in 0..BRANCH_FACTOR {
                        ops.push(op.filter(move |x| x % BRANCH_FACTOR == i))
                    }

                    op = scope.concatenate(ops);
                }

                op.inspect(|i| {
                    black_box(i);
                });
            });
        })
    });
}

fn benchmark_spinach(c: &mut Criterion) {
    c.bench_function("spinach", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| {
            async {
                type MyLatRepr =
                    spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, usize>;
                let op = <spinachflow::op::OnceOp<MyLatRepr>>::new((0..NUM_INTS).collect());

                struct Even();
                impl spinachflow::func::unary::Morphism for Even {
                    type InLatRepr = MyLatRepr;
                    type OutLatRepr = MyLatRepr;
                    fn call<Y: spinachflow::hide::Qualifier>(
                        &self,
                        item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
                    ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
                        item.filter(|i| 0 == i % 2)
                    }
                }

                struct Odds();
                impl spinachflow::func::unary::Morphism for Odds {
                    type InLatRepr = MyLatRepr;
                    type OutLatRepr = MyLatRepr;
                    fn call<Y: spinachflow::hide::Qualifier>(
                        &self,
                        item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
                    ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
                        item.filter(|i| 1 == i % 2)
                    }
                }

                ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
                seq_macro::seq!(N in 0..20 {
                    let [ op_even, op_odds ] = spinachflow::op::fixed_split::<_, 2>(op);
                    let op_even = spinachflow::op::MorphismOp::new(op_even, Even());
                    let op_odds = spinachflow::op::MorphismOp::new(op_odds, Odds());
                    let op = spinachflow::op::MergeOp::new(op_even, op_odds);
                    let op = spinachflow::op::DynOpDelta::new(Box::new(op));
                });

                let comp = spinachflow::comp::NullComp::new(op);
                spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
            }
        });
    });
}

fn benchmark_spinach_switch(c: &mut Criterion) {
    c.bench_function("spinach w/ switch", |b| {
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        )
        .iter(|| {
            async {
                type MyLatRepr =
                    spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, usize>;
                let op = <spinachflow::op::OnceOp<MyLatRepr>>::new((0..NUM_INTS).collect());

                struct SwitchEvenOdd();
                impl spinachflow::func::unary::Morphism for SwitchEvenOdd {
                    type InLatRepr = MyLatRepr;
                    type OutLatRepr = spinachflow::lattice::pair::PairRepr<MyLatRepr, MyLatRepr>;
                    fn call<Y: spinachflow::hide::Qualifier>(
                        &self,
                        item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
                    ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
                        let (a, b) = item.switch(|i| 0 == i % 2);
                        spinachflow::hide::Hide::zip(a, b)
                    }
                }

                ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
                seq_macro::seq!(N in 0..20 {
                    let op = spinachflow::op::MorphismOp::new(op, SwitchEvenOdd());
                    let ( op_even, op_odds ) = spinachflow::op::SwitchOp::new(op);
                    let op = spinachflow::op::MergeOp::new(op_even, op_odds);
                    let op = spinachflow::op::DynOpDelta::new(Box::new(op));
                });

                let comp = spinachflow::comp::NullComp::new(op);
                spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
            }
        });
    });
}

// criterion_group!(
//     name = fork_join_dataflow;
//     config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
//     targets = benchmark_babyflow
// );
// criterion_group!(fork_join_dataflow, benchmark_timely,);
criterion_group!(
    fork_join_dataflow,
    benchmark_babyflow,
    benchmark_timely,
    benchmark_raw,
    benchmark_spinach,
    benchmark_spinach_switch,
);
criterion_main!(fork_join_dataflow);
