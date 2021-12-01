use std::{cell::RefCell, rc::Rc};

use crate::scheduled::collections::Iter;
use crate::scheduled::handoff::VecHandoff;
use crate::scheduled::{Context, Hydroflow, OutputPort, RecvCtx, SendCtx};

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
        let output_port = (*self.df).borrow_mut().add_source(f);
        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn concat<T>(&mut self, ops: Vec<Operator<T>>) -> Operator<T>
    where
        T: 'static,
    {
        let (inputs, output) = (*self.df).borrow_mut().add_n_in_m_out(
            ops.len(),
            1,
            |ins: &[&RecvCtx<VecHandoff<T>>], out| {
                for input in ins {
                    out[0].give(Iter(input.take_inner().into_iter()));
                }
            },
        );

        for (op, input) in ops.into_iter().zip(inputs.into_iter()) {
            (*self.df).borrow_mut().add_edge(op.output_port, input)
        }

        Operator {
            df: self.df.clone(),
            output_port: output.into_iter().next().unwrap(),
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
    pub fn map<U, F>(self, f: F) -> Operator<U>
    where
        F: 'static + Fn(T) -> U,
        U: 'static,
    {
        let (input, output) =
            (*self.df)
                .borrow_mut()
                .add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<T>>, send| {
                    #[allow(clippy::redundant_closure)]
                    send.give(Iter(recv.take_inner().into_iter().map(|x| f(x))));
                });

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        Operator {
            df: self.df,
            output_port: output,
        }
    }

    pub fn filter<F>(self, f: F) -> Operator<T>
    where
        F: 'static + Fn(&T) -> bool,
    {
        let (input, output) =
            (*self.df)
                .borrow_mut()
                .add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<T>>, send| {
                    send.give(Iter(recv.take_inner().into_iter().filter(|x| f(x))));
                });

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        Operator {
            df: self.df,
            output_port: output,
        }
    }

    pub fn concat(self, other: Operator<T>) -> Operator<T> {
        // TODO(justin): this is very slow.
        let (input1, input2, output) = (*self.df).borrow_mut().add_binary(
            |_ctx, recv1: &RecvCtx<VecHandoff<T>>, recv2: &RecvCtx<VecHandoff<T>>, send| {
                send.give(Iter(recv1.take_inner().into_iter()));
                send.give(Iter(recv2.take_inner().into_iter()));
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
        F: 'static + Fn(T),
    {
        let input = (*self.df)
            .borrow_mut()
            .add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<T>>| {
                for v in recv.take_inner() {
                    f(v)
                }
            });

        (*self.df).borrow_mut().add_edge(self.output_port, input);
    }
}

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
