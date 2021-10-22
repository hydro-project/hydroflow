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
    pub fn tee(self) -> (Operator<T>, Operator<T>)
    where
        T: 'static,
    {
        // TODO(justin): this is very slow.
        let (input, output1, output2) = (*self.df).borrow_mut().add_binary_out(
            move |recv: &mut RecvCtx<VecHandoff<T>>, send1, send2| {
                for v in recv.into_iter() {
                    send1.give(Some(v.clone()));
                    send2.give(Some(v));
                }
            },
        );

        (*self.df).borrow_mut().add_edge(self.output_port, input);

        (
            Operator {
                df: self.df.clone(),
                output_port: output1,
            },
            Operator {
                df: self.df,
                output_port: output2,
            },
        )
    }
}
