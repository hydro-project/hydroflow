use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::handoff::VecHandoff;
use hydroflow::scheduled::query::Query as Q;
use timely::dataflow::operators::{Concatenate, Filter, Inspect, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 100_000;
const BRANCH_FACTOR: usize = 2;

fn benchmark_hydroflow(c: &mut Criterion) {
    c.bench_function("fork_join/hydroflow", |b| {
        b.iter(|| {
            let mut df = Hydroflow::new();

            let (start_send, start_recv) = df.make_edge::<VecHandoff<usize>>();

            let mut sent = false;
            df.add_subgraph_source(start_send, move |_ctx, send| {
                if !sent {
                    sent = true;
                    send.give(Iter(0..NUM_INTS));
                }
            });

            let (send1, mut recv1) = df.make_edge::<VecHandoff<_>>();
            let (send2, mut recv2) = df.make_edge::<VecHandoff<_>>();

            df.add_subgraph_in_2out(start_recv, send1, send2, |_ctx, recv, send1, send2| {
                for v in recv.take_inner().into_iter() {
                    if v % 2 == 0 {
                        send1.give(Some(v));
                    } else {
                        send2.give(Some(v));
                    }
                }
            });

            for _ in 0..NUM_OPS {
                let (send1, next_recv1) = df.make_edge();
                let (send2, next_recv2) = df.make_edge();

                df.add_subgraph_2in_2out(
                    recv1,
                    recv2,
                    send1,
                    send2,
                    |_ctx, recv1, recv2, send1, send2| {
                        for v in recv1
                            .take_inner()
                            .into_iter()
                            .chain(recv2.take_inner().into_iter())
                        {
                            if v % 2 == 0 {
                                send1.give(Some(v));
                            } else {
                                send2.give(Some(v));
                            }
                        }
                    },
                );

                recv1 = next_recv1;
                recv2 = next_recv2;
            }

            df.add_subgraph_2sink(recv1, recv2, |_ctx, recv1, recv2| {
                for x in recv1.take_inner() {
                    black_box(x);
                }
                for x in recv2.take_inner() {
                    black_box(x);
                }
            });

            df.tick()
        })
    });
}

fn benchmark_hydroflow_builder(c: &mut Criterion) {
    c.bench_function("fork_join/hydroflow_builder", |b| {
        b.iter(|| {
            // TODO(justin): this creates more operators than necessary.
            let mut q = Q::new();

            let mut source = q.source(|_ctx, send| {
                send.give(Iter(0..NUM_INTS));
            });

            for _ in 0..NUM_OPS {
                let mut outs = source.tee(2).into_iter();
                let (mut out1, mut out2) = (outs.next().unwrap(), outs.next().unwrap());
                out1 = out1.filter(|x| x % 2 == 0);
                out2 = out2.filter(|x| x % 2 == 1);
                source = out1.concat(out2);
            }

            source.sink(|v| {
                black_box(v);
            });

            q.tick();
        })
    });
}

fn benchmark_raw(c: &mut Criterion) {
    c.bench_function("fork_join/raw", |b| {
        b.iter(|| {
            let mut parts = [(); BRANCH_FACTOR].map(|_| Vec::new());
            let mut data: Vec<_> = (0..NUM_INTS).collect();

            for _ in 0..NUM_OPS {
                for i in data.drain(..) {
                    parts[i % BRANCH_FACTOR].push(i);
                }

                for part in parts.iter_mut() {
                    data.append(part);
                }
            }
        })
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("fork_join/timely", |b| {
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

// fn benchmark_spinach(c: &mut Criterion) {
//     c.bench_function("spinach", |b| {
//         b.to_async(
//             tokio::runtime::Builder::new_current_thread()
//                 .build()
//                 .unwrap(),
//         )
//         .iter(|| {
//             async {
//                 use spinachflow::comp::Comp;

//                 type MyLatRepr =
//                     spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, usize>;
//                 let op = <spinachflow::op::OnceOp<MyLatRepr>>::new((0..NUM_INTS).collect());

//                 struct Even();
//                 impl spinachflow::func::unary::Morphism for Even {
//                     type InLatRepr = MyLatRepr;
//                     type OutLatRepr = MyLatRepr;
//                     fn call<Y: spinachflow::hide::Qualifier>(
//                         &self,
//                         item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
//                     ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
//                         item.filter(|i| 0 == i % 2)
//                     }
//                 }

//                 struct Odds();
//                 impl spinachflow::func::unary::Morphism for Odds {
//                     type InLatRepr = MyLatRepr;
//                     type OutLatRepr = MyLatRepr;
//                     fn call<Y: spinachflow::hide::Qualifier>(
//                         &self,
//                         item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
//                     ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
//                         item.filter(|i| 1 == i % 2)
//                     }
//                 }

//                 ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
//                 seq_macro::seq!(N in 0..20 {
//                     let [ op_even, op_odds ] = spinachflow::op::fixed_split::<_, 2>(op);
//                     let op_even = spinachflow::op::MorphismOp::new(op_even, Even());
//                     let op_odds = spinachflow::op::MorphismOp::new(op_odds, Odds());
//                     let op = spinachflow::op::MergeOp::new(op_even, op_odds);
//                     let op = spinachflow::op::DynOpDelta::new(Box::new(op));
//                 });

//                 let comp = spinachflow::comp::NullComp::new(op);
//                 spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
//             }
//         });
//     });
// }

// fn benchmark_spinach_switch(c: &mut Criterion) {
//     c.bench_function("spinach w/ switch", |b| {
//         b.to_async(
//             tokio::runtime::Builder::new_current_thread()
//                 .build()
//                 .unwrap(),
//         )
//         .iter(|| {
//             async {
//                 use spinachflow::comp::Comp;

//                 type MyLatRepr =
//                     spinachflow::lattice::set_union::SetUnionRepr<spinachflow::tag::VEC, usize>;
//                 let op = <spinachflow::op::OnceOp<MyLatRepr>>::new((0..NUM_INTS).collect());

//                 struct SwitchEvenOdd();
//                 impl spinachflow::func::unary::Morphism for SwitchEvenOdd {
//                     type InLatRepr = MyLatRepr;
//                     type OutLatRepr = spinachflow::lattice::pair::PairRepr<MyLatRepr, MyLatRepr>;
//                     fn call<Y: spinachflow::hide::Qualifier>(
//                         &self,
//                         item: spinachflow::hide::Hide<Y, Self::InLatRepr>,
//                     ) -> spinachflow::hide::Hide<Y, Self::OutLatRepr> {
//                         let (a, b) = item.switch(|i| 0 == i % 2);
//                         spinachflow::hide::Hide::zip(a, b)
//                     }
//                 }

//                 ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
//                 seq_macro::seq!(N in 0..20 {
//                     let op = spinachflow::op::MorphismOp::new(op, SwitchEvenOdd());
//                     let ( op_even, op_odds ) = spinachflow::op::SwitchOp::new(op);
//                     let op = spinachflow::op::MergeOp::new(op_even, op_odds);
//                     let op = spinachflow::op::DynOpDelta::new(Box::new(op));
//                 });

//                 let comp = spinachflow::comp::NullComp::new(op);
//                 spinachflow::comp::CompExt::run(&comp).await.unwrap_err();
//             }
//         });
//     });
// }

// fn benchmark_spinachflow_symm(c: &mut Criterion) {
//     c.bench_function("spinachflow (symmetric)", |b| {
//         b.to_async(
//             tokio::runtime::Builder::new_current_thread()
//                 .build()
//                 .unwrap(),
//         )
//         .iter(|| {
//             async {
//                 use spinachflow::futures::StreamExt;
//                 use spinachflow::futures::future::ready;

//                 let stream = spinachflow::futures::stream::iter(0..NUM_INTS);

//                 ///// MAGIC NUMBER!!!!!!!! is NUM_OPS
//                 seq_macro::seq!(N in 0..20 {
//                     let splitter = spinachflow::stream::Splitter::new(stream);
//                     let mut i = 0;
//                     let splits = [(); BRANCH_FACTOR].map(|_| {
//                         let j = i;
//                         i += 1;
//                         splitter.add_split().filter(move |x| ready(j == x % BRANCH_FACTOR))
//                     });
//                     let stream = spinachflow::stream::SelectArr::new(splits);
//                     let stream: std::pin::Pin<Box<dyn spinachflow::futures::Stream<Item = usize>>> = Box::pin(stream);
//                 });

//                 let mut stream = stream;
//                 loop {
//                     let item = stream.next().await;
//                     if item.is_none() {
//                         break;
//                     }
//                 }
//             }
//         });
//     });
// }

// criterion_group!(
//     name = fork_join_dataflow;
//     config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
//     targets = benchmark_babyflow
// );
// criterion_group!(fork_join_dataflow, benchmark_timely,);
criterion_group!(
    fork_join_dataflow,
    benchmark_hydroflow,
    benchmark_hydroflow_builder,
    benchmark_timely,
    benchmark_raw,
    // benchmark_spinach,
    // benchmark_spinach_switch,
    // benchmark_spinachflow_symm,
);
criterion_main!(fork_join_dataflow);
