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

            let source = q.source(|send| {
                send.give_iterator(0..NUM_INTS);
            });

            let _sinks: Vec<_> = (0..NUM_OPS)
                .map(|_| source.clone().map(black_box))
                .collect();

            (*q.df).borrow_mut().run();
        })
    });
}

fn benchmark_timely(c: &mut Criterion) {
    c.bench_function("timely", |b| {
        b.iter(|| {
            timely::example(move |scope| {
                let source = (0..NUM_INTS).to_stream(scope);

                let _sinks: Vec<_> = (0..NUM_OPS)
                    .map(|_| source.clone().map(black_box))
                    .collect();
            });
        })
    });
}

criterion_group!(fan_out_dataflow, benchmark_babyflow, benchmark_timely);
criterion_main!(fan_out_dataflow);
