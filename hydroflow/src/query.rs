use std::{cell::RefCell, rc::Rc};

use crate::{collections::Iter, handoff::VecHandoff, Hydroflow, OutputPort, RecvCtx, SendCtx};

pub struct Query {
    df: Rc<RefCell<Hydroflow>>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            df: Rc::new(RefCell::new(Hydroflow::new())),
        }
    }

    pub fn source<F, T>(&mut self, f: F) -> Operator<T>
    where
        T: 'static,
        F: 'static + FnMut(&mut SendCtx<VecHandoff<T>>),
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
        let (inputs, output) = (*self.df)
            .borrow_mut()
            .add_n_in_m_out(ops.len(), 1, |ins, out| {
                for input in ins {
                    out[0].give(Iter(input.into_iter()));
                }
            });

        for (op, input) in ops.into_iter().zip(inputs.into_iter()) {
            (*self.df).borrow_mut().add_edge(op.output_port, input)
        }

        Operator {
            df: self.df.clone(),
            output_port: output.into_iter().next().unwrap(),
        }
    }

    pub fn run(&mut self) {
        (*self.df).borrow_mut().run();
    }
}

pub struct Operator<T> {
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
        let (input, output) = (*self.df).borrow_mut().add_inout(move |recv, send| {
            send.give(Iter(recv.into_iter().map(|x| f(x))));
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
        let (input, output) = (*self.df).borrow_mut().add_inout(move |recv, send| {
            send.give(Iter(recv.into_iter().filter(|x| f(x))));
        });

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        Operator {
            df: self.df,
            output_port: output,
        }
    }

    pub fn concat(self, other: Operator<T>) -> Operator<T> {
        // TODO(justin): this is very slow.
        let (input1, input2, output) = (*self.df).borrow_mut().add_binary(|recv1, recv2, send| {
            send.give(Iter(recv1.into_iter()));
            send.give(Iter(recv2.into_iter()));
        });
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
        let input = (*self.df).borrow_mut().add_sink(move |recv| {
            for v in recv.into_iter() {
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
            move |recvs: &mut [RecvCtx<VecHandoff<T>>], sends| {
                // TODO(justin): optimize this (extra clone, etc.).
                for v in recvs.into_iter().next().unwrap().into_iter() {
                    for s in &mut *sends {
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
