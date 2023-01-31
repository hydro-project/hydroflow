use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use hydroflow::hydroflow_syntax;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn ops(c: &mut Criterion) {
    let mut rng = StdRng::from_entropy();

    c.bench_function("micro/ops/identity", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let data: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    source_iter(black_box(data)) -> identity() -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/unique", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let data: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    source_iter(data) -> unique() -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/map", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let data: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    source_iter(black_box(data)) -> map(|x| x + 1) -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/flat_map", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let data: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    source_iter(black_box(data)) -> flat_map(|x| [x]) -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/join", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_join = join();

                    source_iter(black_box(input0)) -> [0]my_join;
                    source_iter(black_box(input1)) -> [1]my_join;

                    my_join -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/difference", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_difference = difference();

                    source_iter(black_box(input0)) -> [pos]my_difference;
                    source_iter(black_box(input1)) -> [neg]my_difference;

                    my_difference -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/merge", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();
                let input1: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    my_merge = merge();

                    source_iter(black_box(input0)) -> my_merge;
                    source_iter(black_box(input1)) -> my_merge;

                    my_merge -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/tee", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    my_tee = tee();

                    source_iter(black_box(input0)) -> my_tee;

                    my_tee -> for_each(|x| { black_box(x); });
                    my_tee -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/fold", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                #[allow(clippy::unnecessary_fold)]
                {
                    hydroflow_syntax! {
                        source_iter(black_box(input0)) -> fold::<'tick>(0, |accum, elem| { accum + elem }) -> for_each(|x| { black_box(x); });
                    }
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/sort", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 10_000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    source_iter(black_box(input0)) -> sort() -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("micro/ops/crossjoin", |b| {
        b.iter_batched_ref(
            || {
                const NUM_INTS: usize = 1000;
                let dist = Uniform::new(0, 100);
                let input0: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();
                let input1: Vec<usize> = (0..NUM_INTS).map(|_| dist.sample(&mut rng)).collect();

                hydroflow_syntax! {
                    my_crossjoin = cross_join();

                    source_iter(black_box(input0)) -> [0]my_crossjoin;
                    source_iter(black_box(input1)) -> [1]my_crossjoin;

                    my_crossjoin -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(micro_ops, ops,);
criterion_main!(micro_ops);
