use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hydroflow::{Hydroflow, RecvCtx, SendCtx, VecHandoff};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;

fn benchmark_identity(c: &mut Criterion) {
    c.bench_function("identity", |b| {
        b.iter(|| {
            let mut df = Hydroflow::new();

            let mut sent = false;
            let mut it = df.add_source(move |send: &mut SendCtx<VecHandoff<_>>| {
                if !sent {
                    sent = true;
                    for x in 0..NUM_INTS {
                        send.try_give(x).unwrap();
                    }
                }
            });
            for _ in 0..NUM_OPS {
                let (next_in, mut next_out) = df.add_inout(|recv, send| {
                    for x in &*recv {
                        send.try_give(x).unwrap();
                    }
                });

                std::mem::swap(&mut it, &mut next_out);
                df.add_edge(next_out, next_in);
            }

            let sink = df.add_sink(|recv: &mut RecvCtx<VecHandoff<usize>>| {
                for x in &*recv {
                    black_box(x);
                }
            });
            df.add_edge(it, sink);

            df.run();
        });
    });
}

criterion_group!(benches, benchmark_identity);
criterion_main!(benches);
