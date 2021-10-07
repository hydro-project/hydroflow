#![allow(dead_code, unused_imports)]

use babyflow::babyflow::{Operator, Query};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use timely::dataflow::operators::{Concat, Concatenate, Filter, Inspect, Map, ToStream};

const NUM_OPS: usize = 20;
const NUM_INTS: usize = 1_000_000;

fn benchmark_babyflow(c: &mut Criterion) {
    c.bench_function("babyflow", |b| {
        b.iter(|| {
            let mut q = Query::new();

            let sources: Vec<_> = (0..NUM_OPS)
                .map(|i| {
                    q.source(move |send| {
                        send.give_iterator((i * NUM_INTS)..((i + 1) * NUM_INTS));
                    })
                })
                .collect();

            let op = q.concat(sources);

            let sum = Rc::new(RefCell::new(0));
            let c = sum.clone();
            op.sink(move |v| {
                *(*c).borrow_mut() += v;
            });

            (*q.df).borrow_mut().run();
        })
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("timely", |b| {
        b.iter(|| {
            timely::example(move |scope| {
                let sources: Vec<_> = (0..NUM_OPS)
                    .map(|i| ((i * NUM_INTS)..((i + 1) * NUM_INTS)).to_stream(scope))
                    .collect();

                let merged = scope.concatenate(sources);

                merged.inspect(|x| {
                    black_box(x);
                });
            });
        })
    });
}

criterion_group!(fan_in_dataflow, benchmark_babyflow, benchmark_timely);
criterion_main!(fan_in_dataflow);
