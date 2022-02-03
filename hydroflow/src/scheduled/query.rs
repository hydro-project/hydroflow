use std::{cell::RefCell, rc::Rc};

use crate::lang::collections::Iter;
use crate::scheduled::graph::Hydroflow;
use crate::scheduled::handoff::VecHandoff;

use super::context::Context;
use super::graph_ext::GraphExt;
use super::port::{OutputPort, RecvCtx, SendCtx};

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
        let mut df = self.df.borrow_mut();

        let (send_port, recv_port) = df.make_handoff();
        df.add_subgraph_source(send_port, f);

        Operator {
            df: self.df.clone(),
            recv_port,
        }
    }

    pub fn concat<T>(&mut self, ops: Vec<Operator<T>>) -> Operator<T>
    where
        T: 'static,
    {
        let mut df = self.df.borrow_mut();

        let (send_port, recv_port) = df.make_handoff();
        df.add_subgraph_homogeneous(
            ops.into_iter().map(|op| op.recv_port).collect(),
            vec![send_port],
            |_ctx, ins, out| {
                for &input in ins {
                    out[0].give(Iter(input.take_inner().into_iter()));
                }
            },
        );

        Operator {
            df: self.df.clone(),
            recv_port,
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
    recv_port: OutputPort<VecHandoff<T>>,
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
        let mut df = self.df.borrow_mut();

        let (send_port, recv_port) = df.make_handoff();
        df.add_subgraph_in_out(self.recv_port, send_port, move |_ctx, recv, send| {
            send.give(Iter(recv.take_inner().into_iter().map(&mut f)));
        });

        std::mem::drop(df);
        Operator {
            df: self.df,
            recv_port,
        }
    }

    #[must_use]
    pub fn filter<F>(self, mut f: F) -> Operator<T>
    where
        F: 'static + Fn(&T) -> bool,
    {
        let mut df = self.df.borrow_mut();

        let (send_port, recv_port) = df.make_handoff();
        df.add_subgraph_in_out(self.recv_port, send_port, move |_ctx, recv, send| {
            send.give(Iter(recv.take_inner().into_iter().filter(&mut f)));
        });

        std::mem::drop(df);
        Operator {
            df: self.df,
            recv_port,
        }
    }

    #[must_use]
    pub fn concat(self, other: Operator<T>) -> Operator<T> {
        // TODO(justin): this is very slow.

        let mut df = self.df.borrow_mut();

        let (send_port, recv_port) = df.make_handoff::<VecHandoff<T>>();
        df.add_subgraph_2in_out(
            self.recv_port,
            other.recv_port,
            send_port,
            |_ctx, recv1, recv2, send| {
                send.give(Iter(recv1.take_inner().into_iter()));
                send.give(Iter(recv2.take_inner().into_iter()));
            },
        );

        std::mem::drop(df);
        Operator {
            df: self.df,
            recv_port,
        }
    }

    pub fn sink<F>(self, f: F)
    where
        F: 'static + Fn(T),
    {
        self.df.borrow_mut().add_subgraph_sink(
            self.recv_port,
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
