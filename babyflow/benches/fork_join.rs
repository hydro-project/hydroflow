#![allow(dead_code, unused_imports)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use datalog::babyflow::{Operator, Query};
use pprof::criterion::{Output, PProfProfiler};
use std::sync::mpsc::channel;
use std::thread::{self, sleep};
use std::time::Duration;
use timely::dataflow::operators::{Concat, Concatenate, Filter, Inspect, Map, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;
const BRANCH_FACTOR: usize = 2;

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
                use spinach::comp::Comp;

                type MyLatRepr =
                    spinach::lattice::set_union::SetUnionRepr<spinach::tag::VEC, usize>;
                let op = <spinach::op::OnceOp<MyLatRepr>>::new((0..NUM_INTS).collect());

                struct Even();
                impl spinach::func::unary::Morphism for Even {
                    type InLatRepr = MyLatRepr;
                    type OutLatRepr = MyLatRepr;
                    fn call<Y: spinach::hide::Qualifier>(
                        &self,
                        item: spinach::hide::Hide<Y, Self::InLatRepr>,
                    ) -> spinach::hide::Hide<Y, Self::OutLatRepr> {
                        item.filter(|i| 0 == i % 2)
                    }
                }

                struct Odds();
                impl spinach::func::unary::Morphism for Odds {
                    type InLatRepr = MyLatRepr;
                    type OutLatRepr = MyLatRepr;
                    fn call<Y: spinach::hide::Qualifier>(
                        &self,
                        item: spinach::hide::Hide<Y, Self::InLatRepr>,
                    ) -> spinach::hide::Hide<Y, Self::OutLatRepr> {
                        item.filter(|i| 1 == i % 2)
                    }
                }

                ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
                seq_macro::seq!(N in 0..20 {
                    let [ op_even, op_odds ] = spinach::op::fixed_split::<_, 2>(op);
                    let op_even = spinach::op::MorphismOp::new(op_even, Even());
                    let op_odds = spinach::op::MorphismOp::new(op_odds, Odds());
                    let op = spinach::op::MergeOp::new(op_even, op_odds);
                    let op = spinach::op::DynOpDelta::new(Box::new(op));
                });

                let comp = spinach::comp::NullComp::new(op);
                spinach::comp::CompExt::run(&comp).await.unwrap_err();
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
                use spinach::comp::Comp;

                type MyLatRepr =
                    spinach::lattice::set_union::SetUnionRepr<spinach::tag::VEC, usize>;
                let op = <spinach::op::OnceOp<MyLatRepr>>::new((0..NUM_INTS).collect());

                struct SwitchEvenOdd();
                impl spinach::func::unary::Morphism for SwitchEvenOdd {
                    type InLatRepr = MyLatRepr;
                    type OutLatRepr = spinach::lattice::pair::PairRepr<MyLatRepr, MyLatRepr>;
                    fn call<Y: spinach::hide::Qualifier>(
                        &self,
                        item: spinach::hide::Hide<Y, Self::InLatRepr>,
                    ) -> spinach::hide::Hide<Y, Self::OutLatRepr> {
                        let (a, b) = item.switch(|i| 0 == i % 2);
                        spinach::hide::Hide::zip(a, b)
                    }
                }

                ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
                seq_macro::seq!(N in 0..20 {
                    let op = spinach::op::MorphismOp::new(op, SwitchEvenOdd());
                    let ( op_even, op_odds ) = spinach::op::SwitchOp::new(op);
                    let op = spinach::op::MergeOp::new(op_even, op_odds);
                    let op = spinach::op::DynOpDelta::new(Box::new(op));
                });

                let comp = spinach::comp::NullComp::new(op);
                spinach::comp::CompExt::run(&comp).await.unwrap_err();
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
    benchmark_spinach,
    benchmark_spinach_switch,
);
criterion_main!(fork_join_dataflow);
