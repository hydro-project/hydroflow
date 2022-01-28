use std::{cell::RefCell, rc::Rc};

use crate::lang::collections::Iter;
use crate::scheduled::graph::Hydroflow;
use crate::scheduled::handoff::VecHandoff;
use crate::{tl, tt};

use super::context::Context;
use super::port::{OutputPort, RecvCtx, SendCtx};
use super::graph_ext::GraphExt;

#[derive(Default)]
pub struct Query {
    df: Rc<RefCell<Hydroflow>>,
}

impl Query {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn source<F, T>(&mut self, f: F) -> Operator<T>
    where
        T: 'static,
        F: 'static + FnMut(&Context<'_>, &SendCtx<VecHandoff<T>>),
    {
        let output_port = self.df.borrow_mut().add_source(f);
        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn concat<T>(&mut self, ops: Vec<Operator<T>>) -> Operator<T>
    where
        T: 'static,
    {
        let mut df = self.df.borrow_mut();

        let (input_port, output_port) = df.make_handoff();

        df.add_subgraph_homogeneous(
            ops.into_iter().map(|op| op.output_port).collect(),
            vec![input_port],
            |ins, out| {
                for &input in ins {
                    out[0].give(Iter(input.take_inner().into_iter()));
                }
            },
        );

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn tick(&mut self) {
        (*self.df).borrow_mut().tick()
    }
}

pub struct Operator<T>
where
    T: 'static,
{
    df: Rc<RefCell<Hydroflow>>,
    output_port: OutputPort<VecHandoff<T>>,
}

impl<T> Operator<T>
where
    T: 'static,
{
    pub fn map<U, F>(self, mut f: F) -> Operator<U>
    where
        F: 'static + Fn(T) -> U,
        U: 'static,
    {
        let output_port =
            self.df
                .borrow_mut()
                .add_inout(self.output_port, move |_ctx, recv, send| {
                    send.give(Iter(recv.take_inner().into_iter().map(&mut f)));
                });

        Operator {
            df: self.df,
            output_port,
        }
    }

    #[must_use]
    pub fn filter<F>(self, mut f: F) -> Operator<T>
    where
        F: 'static + Fn(&T) -> bool,
    {
        let output_port =
            self.df
                .borrow_mut()
                .add_inout(self.output_port, move |_ctx, recv, send| {
                    send.give(Iter(recv.take_inner().into_iter().filter(&mut f)));
                });

        Operator {
            df: self.df,
            output_port,
        }
    }

    #[must_use]
    pub fn concat(self, other: Operator<T>) -> Operator<T> {
        // TODO(justin): this is very slow.

        let output_port = self.df.borrow_mut().add_binary(
            self.output_port,
            other.output_port,
            |_ctx, recv1: &RecvCtx<VecHandoff<T>>, recv2: &RecvCtx<VecHandoff<T>>, send| {
                send.give(Iter(recv1.take_inner().into_iter()));
                send.give(Iter(recv2.take_inner().into_iter()));
            },
        );

        Operator {
            df: self.df,
            output_port,
        }
    }

    pub fn sink<F>(self, f: F)
    where
        F: 'static + Fn(T),
    {
        self.df.borrow_mut().add_sink(
            self.output_port,
            move |_ctx, recv: &RecvCtx<VecHandoff<T>>| {
                for v in recv.take_inner() {
                    f(v)
                }
            },
        );
    }
}

/*
impl<T: Clone> Operator<T> {
    pub fn tee(self, n: usize) -> Vec<Operator<T>>
    where
        T: 'static,
    {
        // TODO(justin): this is very slow.
        let (inputs, outputs) = (*self.df).borrow_mut().add_n_in_m_out(
            1,
            n,
            move |recvs: &[&RecvCtx<VecHandoff<T>>], sends| {
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
*/
