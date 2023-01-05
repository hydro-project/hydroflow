use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use hydroflow::hydroflow_syntax;
use hydroflow::util::unbounded_channel;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn ops(c: &mut Criterion) {
    let mut rng = StdRng::from_entropy();

    c.bench_function("hydroflow/micro/ops/identity", |b| {
        const NUM_INTS: usize = 300_000;
        let dist = Uniform::new(0, 100);

        let (s1, r1) = unbounded_channel::<(usize, ())>();

        let mut df = hydroflow_syntax! {
            input1 = source_stream(r1);

            input1 -> identity() -> for_each(|x| { black_box(x); });
        };

        b.iter_batched(
            || {
                for _ in 0..NUM_INTS {
                    s1.send((black_box(dist.sample(&mut rng)), ())).unwrap();
                }
            },
            |()| {
                df.run_available();
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("hydroflow/micro/ops/unique", |b| {
        const NUM_INTS: usize = 300_000;
        let dist = Uniform::new(0, 100);

        let (s1, r1) = unbounded_channel::<(usize, ())>();

        let mut df = hydroflow_syntax! {
            input1 = source_stream(r1);

            input1 -> unique() -> for_each(|x| { black_box(x); });
        };

        b.iter_batched(
            || {
                for _ in 0..NUM_INTS {
                    s1.send((black_box(dist.sample(&mut rng)), ())).unwrap();
                }
            },
            |()| {
                df.run_available();
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("hydroflow/micro/ops/join", |b| {
        const NUM_INTS: usize = 100_000;
        let dist = Uniform::new(0, 100);

        let (s1, r1) = unbounded_channel::<(usize, ())>();
        let (s2, r2) = unbounded_channel::<(usize, ())>();

        let mut df = hydroflow_syntax! {
            input1 = source_stream(r1);
            input2 = source_stream(r2);

            my_join = join();
            input1 -> [0]my_join;
            input2 -> [1]my_join;

            my_join -> for_each(|x| { black_box(x); });
        };

        b.iter_batched(
            || {
                for _ in 0..NUM_INTS {
                    s1.send((black_box(dist.sample(&mut rng)), ())).unwrap();
                    s2.send((black_box(dist.sample(&mut rng)), ())).unwrap();
                }
            },
            |()| {
                df.run_available();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(micro_ops, ops,);
criterion_main!(micro_ops);
