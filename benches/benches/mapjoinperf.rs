use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use hydroflow::hydroflow_syntax;
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};
use std::hint::black_box;

fn ops(c: &mut Criterion) {
    const NUM_INTS: usize = 10_000;
    let mut rng = StdRng::from_entropy();

    c.bench_function("mapjoinperf/usize_nomapinput_nomapoutput", |b| {
        b.iter_batched_ref(
            || {
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, (usize, usize))> = (0..NUM_INTS)
                    .map(|_| {
                        (
                            dist.sample(&mut rng),
                            (dist.sample(&mut rng), dist.sample(&mut rng)),
                        )
                    })
                    .collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_join = join();

                    source_iter(black_box(input0))
                        -> [0]my_join;

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

    c.bench_function("mapjoinperf/usize_mapinput_nomapoutput", |b| {
        b.iter_batched_ref(
            || {
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, usize, usize)> = (0..NUM_INTS)
                    .map(|_| {
                        (
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                        )
                    })
                    .collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_join = join();

                    source_iter(black_box(input0))
                        -> map(|(a, b, c)| (a, (b, c)))
                        -> [0]my_join;

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

    c.bench_function("mapjoinperf/usize_nomapinput_mapoutput", |b| {
        b.iter_batched_ref(
            || {
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, (usize, usize))> = (0..NUM_INTS)
                    .map(|_| {
                        (
                            dist.sample(&mut rng),
                            (dist.sample(&mut rng), dist.sample(&mut rng)),
                        )
                    })
                    .collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_join = join();

                    source_iter(black_box(input0))
                        -> [0]my_join;

                    source_iter(black_box(input1)) -> [1]my_join;

                    my_join
                        -> map(|(a, ((b, c), ()))| (a, b, c, ()))
                        -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("mapjoinperf/usize_mapinput_mapoutput", |b| {
        b.iter_batched_ref(
            || {
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, usize, usize)> = (0..NUM_INTS)
                    .map(|_| {
                        (
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                        )
                    })
                    .collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_join = join();

                    source_iter(black_box(input0))
                        -> map(|(a, b, c)| (a, (b, c)))
                        -> [0]my_join;

                    source_iter(black_box(input1)) -> [1]my_join;

                    my_join
                        -> map(|(a, ((b, c), ()))| (a, b, c, ()))
                        -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("mapjoinperf/usize_nomapinput_dupoutput", |b| {
        b.iter_batched_ref(
            || {
                let dist = Uniform::new(0, 100);
                let input0: Vec<(usize, (usize, usize))> = (0..NUM_INTS)
                    .map(|_| {
                        (
                            dist.sample(&mut rng),
                            (dist.sample(&mut rng), dist.sample(&mut rng)),
                        )
                    })
                    .collect();
                let input1: Vec<(usize, ())> =
                    (0..NUM_INTS).map(|_| (dist.sample(&mut rng), ())).collect();

                hydroflow_syntax! {
                    my_join = join();

                    source_iter(black_box(input0))
                        -> [0]my_join;

                    source_iter(black_box(input1)) -> [1]my_join;

                    my_join
                        -> map(|(a, ((b, c), ()))| (a, b, c, c))
                        -> for_each(|x| { black_box(x); });
                }
            },
            |df| {
                df.run_available();
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(mapjoinperf, ops,);
criterion_main!(mapjoinperf);
