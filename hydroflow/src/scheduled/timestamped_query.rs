use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use crate::{
    lang::Lattice,
    progress::{self, Frontier, Timestamp},
    scheduled::{collections::Iter, Hydroflow, OutputPort, RecvCtx, SendCtx},
};

use super::{
    handoff::{BatchedHandoff, TimestampedBatch},
    input::{Buffer, Input},
    Context,
};

// This file implements a timestamped version of Hydroflow on top of the
// existing system. More used as a proof-of-concept, since it does not integrate
// with compiled components particularly well.

type TimeHandoff<T, K, V> = BatchedHandoff<(K, V), TimestampedBatch<T, (K, V)>>;
type TimeInput<T, D> =
    Input<Option<TimestampedBatch<T, D>>, Buffer<Option<TimestampedBatch<T, D>>>>;

#[derive(Default)]
pub struct TimeQuery {
    df: Rc<RefCell<Hydroflow>>,
}

pub struct Operator<T, D>
where
    T: 'static + Timestamp,
    D: 'static,
{
    df: Rc<RefCell<Hydroflow>>,
    output_port: OutputPort<BatchedHandoff<D, TimestampedBatch<T, D>>>,
}

impl TimeQuery {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source<F, D, T>(&mut self, f: F) -> Operator<T, D>
    where
        D: 'static,
        T: Timestamp,
        F: 'static + FnMut(&Context<'_>, &SendCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>),
    {
        let output_port = (*self.df).borrow_mut().add_source(f);
        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn input<T, D>(&mut self) -> (TimeInput<T, D>, Operator<T, D>)
    where
        T: Timestamp,
    {
        let (input, output_port) = (*self.df).borrow_mut().add_input();
        (
            input,
            Operator {
                df: self.df.clone(),
                output_port,
            },
        )
    }

    pub fn tick(&mut self) {
        (*self.df).borrow_mut().tick()
    }
}

struct Reduction<T, K, V> {
    data: BTreeMap<T, HashMap<K, V>>,
    watermark: Frontier<T>,
}

impl<T, K, V> Reduction<T, K, V>
where
    T: Timestamp + std::fmt::Debug,
    V: Lattice + Clone + std::fmt::Debug,
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
    fn new() -> Self {
        Reduction {
            data: BTreeMap::new(),
            watermark: Frontier::new(None),
        }
    }

    fn add(&mut self, t: T, k: K, mut v: V) {
        let set = self.data.entry(t).or_insert_with(HashMap::new);
        let new_val = match set.get(&k) {
            None => v,
            Some(v2) => {
                v.join(v2);
                v
            }
        };
        set.insert(k, new_val);
    }

    // When we get a notification that a timestamp is closed, iterate through
    // all the timestamps that come before it and emit the values at them, then
    // delete them and place their accumulated values at the now-watermark.
    // TODO(justin): better api here that doesn't involve a bunch of allocations.
    fn close(&mut self, watermark: T) -> Vec<(T, Vec<(K, V)>)> {
        let mut notifications = Vec::new();
        let mut accumulated_values = HashMap::new();
        let mut to_remove = Vec::new();
        for (t, data) in self.data.iter() {
            if progress::Timestamp::le(t, &watermark) {
                // This timestamp will now never receive a new write, so we can
                // safely emit the values for each key.
                let mut notifs = Vec::new();
                for (k, v) in data {
                    let mut new_v = v.clone();
                    if let Some(v2) = accumulated_values.get(k) {
                        new_v.join(v2);
                        notifs.push((k.clone(), new_v.clone()));
                    } else {
                        notifs.push((k.clone(), new_v.clone()));
                        accumulated_values.insert(k.clone(), new_v);
                    }
                }
                // If our watermark dominates t, its value has already been emitted.
                if !notifs.is_empty() && !self.watermark.dominates(t) {
                    notifications.push((t.clone(), notifs));
                }
                to_remove.push(t.clone());
            } else {
                // Since the BTree is ordered, and we are assuming total order
                // of timestamps here, we can stop iteration.
                break;
            }
        }
        self.watermark.join_in(&watermark);
        for t in to_remove {
            self.data.remove(&t);
        }
        for (k, v) in accumulated_values.into_iter() {
            self.add(watermark.clone(), k.clone(), v);
        }
        notifications
    }
}

impl<T, K, V> Operator<T, (K, V)>
where
    T: Timestamp + std::fmt::Debug,
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    V: Lattice + Eq + Clone + std::fmt::Debug,
{
    pub fn group_by(self) -> Operator<T, (K, V)> {
        // TODO(justin): For now we assume total ordering of timestamps.
        let mut reduction = Reduction::new();

        let (input, output) = (*self.df).borrow_mut().add_inout(
            move |_ctx: &Context<'_>, recv: &RecvCtx<TimeHandoff<T, K, V>>, send| {
                for batch in recv.take_inner() {
                    for (k, v) in batch.data {
                        reduction.add(batch.timestamp.clone(), k, v);
                    }

                    if let Some(t) = batch.watermark.get() {
                        // TODO(justin): maybe merge these last two batches?
                        for (t, data) in reduction.close(t.clone()) {
                            send.give(Some(TimestampedBatch {
                                data,
                                timestamp: t,
                                watermark: Frontier::new(None),
                            }));
                        }
                        send.give(Some(TimestampedBatch {
                            timestamp: t.clone(),
                            data: Vec::new(),
                            watermark: Frontier::new(Some(t)),
                        }));
                    }
                }
            },
        );

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        Operator {
            df: self.df,
            output_port: output,
        }
    }
}

impl<T, D> Operator<T, D>
where
    T: Timestamp,
    D: 'static,
{
    pub fn map<U, F>(self, f: F) -> Operator<T, U>
    where
        F: 'static + Fn(D) -> U,
        U: 'static,
    {
        let (input, output) = (*self.df).borrow_mut().add_inout(
            move |_ctx: &Context<'_>,
                  recv: &RecvCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>,
                  send| {
                #[allow(clippy::redundant_closure)]
                send.give(Iter(
                    recv.take_inner()
                        .drain(..)
                        .map(|v| v.flat_map(|d| Some(f(d)).into_iter())),
                ));
            },
        );

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        Operator {
            df: self.df,
            output_port: output,
        }
    }

    pub fn filter<F>(self, f: F) -> Operator<T, D>
    where
        F: 'static + Fn(&D) -> bool,
    {
        let (input, output) = (*self.df).borrow_mut().add_inout(
            move |_ctx: &Context<'_>,
                  recv: &RecvCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>,
                  send| {
                send.give(Iter(recv.take_inner().drain(..).map(|mut v| {
                    TimestampedBatch::new(v.timestamp, v.data.drain(..).filter(|x| f(x)).collect())
                })));
            },
        );

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        Operator {
            df: self.df,
            output_port: output,
        }
    }

    pub fn concat(self, other: Operator<T, D>) -> Operator<T, D> {
        // TODO(justin): this is slow: we need to keep track of the
        // frontier of the two inputs and meet them.
        let mut input1_frontier = Frontier::new(None);
        let mut input2_frontier = Frontier::new(None);
        let mut meet = Frontier::new(None);
        let (input1, input2, output) = (*self.df).borrow_mut().add_binary(
            move |_ctx: &Context<'_>,
                  recv1: &RecvCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>,
                  recv2: &RecvCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>,
                  send| {
                for b in recv1.take_inner() {
                    input1_frontier.join_with(&b.watermark);
                    send.give(Some(b));
                }
                for b in recv2.take_inner() {
                    input2_frontier.join_with(&b.watermark);
                    send.give(Some(b));
                }
                let new_meet = input1_frontier.meet(&input2_frontier);
                if new_meet != meet {
                    meet = new_meet;
                    send.give(Some(TimestampedBatch::close(meet.clone())));
                }
            },
        );
        (*self.df).borrow_mut().add_edge(self.output_port, input1);
        (*self.df).borrow_mut().add_edge(other.output_port, input2);

        Operator {
            df: self.df,
            output_port: output,
        }
    }

    pub fn sink<F>(self, f: F)
    where
        F: 'static + Fn(&T, D),
    {
        let input = (*self.df).borrow_mut().add_sink(
            move |_ctx: &Context<'_>, recv: &RecvCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>| {
                for v in recv.take_inner() {
                    for d in v.data {
                        f(&v.timestamp, d)
                    }
                }
            },
        );

        (*self.df).borrow_mut().add_edge(self.output_port, input);
    }
}

impl<D: Clone, T: Timestamp> Operator<T, D> {
    pub fn tee(self, n: usize) -> Vec<Operator<T, D>>
    where
        D: 'static,
    {
        // TODO(justin): this is very slow.
        let (inputs, outputs) = (*self.df).borrow_mut().add_n_in_m_out(
            1,
            n,
            move |recvs: &[&RecvCtx<BatchedHandoff<D, TimestampedBatch<T, D>>>], sends| {
                // TODO(justin): optimize this (extra clone, etc.).
                #[allow(clippy::into_iter_on_ref)]
                for v in recvs.into_iter().next().unwrap().take_inner() {
                    for s in sends {
                        s.give(Some(v.clone()));
                    }
                }
            },
        );

        (*self.df)
            .borrow_mut()
            .add_edge(self.output_port, inputs.into_iter().next().unwrap());

        outputs
            .into_iter()
            .map(|output_port| Operator {
                df: self.df.clone(),
                output_port,
            })
            .collect()
    }
}

#[test]
fn test_ts_query() {
    use crate::lang::Max;

    let mut q = TimeQuery::new();

    let (input, nums) = q.input();

    input.give(Some(TimestampedBatch::new(0, vec![1, 2, 3, 4])));
    input.give(Some(TimestampedBatch::new(0, vec![5, 6, 7, 8]).closing(0)));
    input.give(Some(TimestampedBatch::new(2, vec![1, 2, 3, 4]).closing(2)));

    let result = Rc::new(RefCell::new(Vec::new()));
    let inner = result.clone();
    nums.map(|x| ((), Max::new(x * 2)))
        .group_by()
        .sink(move |t, v| {
            (*inner).borrow_mut().push((*t, v));
        });

    (*q.df).borrow_mut().tick();

    let v = (*result).borrow().clone();
    assert_eq!(v, vec![(0, ((), Max::new(16))), (2, ((), Max::new(16))),])
}
