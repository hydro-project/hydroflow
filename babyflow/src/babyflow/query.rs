use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::babyflow::{Dataflow, InputPort, RecvCtx, SendCtx};

#[derive(Clone)]
pub struct Operator<T>
where
    T: Clone,
{
    df: Rc<RefCell<Dataflow>>,
    output_port: SendCtx<T>,
}

impl<T> Operator<T>
where
    T: Clone,
{
    pub fn distinct(self) -> Operator<T>
    where
        T: Eq + std::hash::Hash + 'static,
    {
        let mut df = (*self.df).borrow_mut();
        let mut tab = HashSet::new();
        let (input, output_port) = df.add_op(move |recv: &RecvCtx<T>, send| {
            while let Some(v) = recv.pull() {
                if !tab.contains(&v) {
                    tab.insert(v.clone());
                    send.push(v)
                }
            }
        });
        df.add_edge(self.output_port.clone(), input);

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    #[allow(unused)]
    pub fn union(self, rhs: Operator<T>) -> Operator<T>
    where
        T: 'static,
    {
        let mut df = (*self.df).borrow_mut();
        let (input1, input2, output_port) = df.add_op_2(move |recv1, recv2, send| {
            send.give_vec(&mut recv1.take_all());
            send.give_vec(&mut recv2.take_all());
        });
        df.add_edge(self.output_port.clone(), input1);
        df.add_edge(rhs.output_port, input2);

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn filter<F>(self, f: F) -> Operator<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let mut df = (*self.df).borrow_mut();
        let (input, output_port) = df.add_op(move |recv, send| {
            let mut vec = recv.take_all();
            vec.retain(|x| f(x));
            send.give_vec(&mut vec);
        });
        df.add_edge(self.output_port.clone(), input);

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn map<U, F>(self, f: F) -> Operator<U>
    where
        F: Fn(T) -> U + 'static,
        T: 'static,
        U: Clone + 'static,
    {
        let mut df = (*self.df).borrow_mut();
        let (input, output_port) = df.add_op(move |recv, send| {
            send.give_iterator(recv.take_all().drain(..).map(&f));
        });
        df.add_edge(self.output_port.clone(), input);

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn sink<F>(self, f: F)
    where
        F: Fn(T) + 'static,
        T: Clone + 'static,
    {
        let mut df = (*self.df).borrow_mut();
        let input = df.add_sink(move |recv| {
            while let Some(v) = recv.pull() {
                f(v)
            }
        });
        df.add_edge(self.output_port.clone(), input);
    }

    pub fn zip<U>(self, o: Operator<U>) -> Operator<(T, U)>
    where
        U: 'static + Clone,
        T: 'static,
    {
        enum State<T, U> {
            None,
            Left(T),
            Right(U),
        }

        let mut df = (*self.df).borrow_mut();
        let mut cur_state = State::None;
        let (in1, in2, output_port) =
            df.add_op_2(move |lhs: &RecvCtx<T>, rhs: &RecvCtx<U>, out| loop {
                match &cur_state {
                    State::None => match (lhs.pull(), rhs.pull()) {
                        (None, None) => {
                            return;
                        }
                        (Some(l), None) => {
                            cur_state = State::Left(l);
                            return;
                        }
                        (None, Some(r)) => {
                            cur_state = State::Right(r);
                            return;
                        }
                        (Some(l), Some(r)) => out.push((l, r)),
                    },
                    State::Left(l) => match rhs.pull() {
                        None => {
                            return;
                        }
                        // TODO: this clone seems sketchy.
                        Some(r) => out.push((l.clone(), r)),
                    },
                    State::Right(r) => match lhs.pull() {
                        None => {
                            return;
                        }
                        Some(l) => out.push((l, r.clone())),
                    },
                }
            });

        df.add_edge(self.output_port, in1);
        df.add_edge(o.output_port, in2);

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }
}

impl<K, V> Operator<(K, V)>
where
    K: Eq + std::hash::Hash + Clone + 'static,
    V: Clone + 'static,
{
    pub fn join<V2>(self, rhs: Operator<(K, V2)>) -> Operator<(K, V, V2)>
    where
        V2: Clone + 'static,
    {
        let mut df = (*self.df).borrow_mut();

        let mut left_tab: HashMap<K, Vec<V>> = HashMap::new();
        let mut right_tab: HashMap<K, Vec<V2>> = HashMap::new();

        let (input1, input2, output_port) = df.add_op_2(
            move |left: &RecvCtx<(K, V)>, right: &RecvCtx<(K, V2)>, send| {
                while let Some((k, v)) = left.pull() {
                    left_tab
                        .entry(k.clone())
                        .or_insert_with(Vec::new)
                        .push(v.clone());
                    if let Some(matches) = right_tab.get(&k) {
                        for v2 in matches {
                            send.push((k.clone(), v.clone(), v2.clone()));
                        }
                    }
                }

                while let Some((k, v)) = right.pull() {
                    right_tab
                        .entry(k.clone())
                        .or_insert_with(Vec::new)
                        .push(v.clone());
                    if let Some(matches) = left_tab.get(&k) {
                        for v2 in matches {
                            send.push((k.clone(), v2.clone(), v.clone()));
                        }
                    }
                }
            },
        );

        df.add_edge(self.output_port.clone(), input1);
        df.add_edge(rhs.output_port, input2);

        Operator {
            df: self.df.clone(),
            output_port,
        }
    }
}

#[derive(Default)]
pub struct Query {
    pub df: Rc<RefCell<Dataflow>>,
}
impl Query {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn wire<T>(&mut self, o: Operator<T>, p: InputPort<T>)
    where
        T: Clone + 'static,
    {
        (*self.df).borrow_mut().add_edge(o.output_port, p)
    }

    pub fn concat<T, I>(&mut self, ops: I) -> Operator<T>
    where
        T: Clone + 'static,
        I: IntoIterator<Item = Operator<T>>,
    {
        let (p, out) = self.merge();
        for o in ops {
            self.wire(o, p.clone())
        }
        out
    }

    pub fn source<T, F>(&mut self, f: F) -> Operator<T>
    where
        T: Clone + 'static,
        F: FnMut(&SendCtx<T>) + 'static,
    {
        let output_port = (*self.df).borrow_mut().add_source(f);
        Operator {
            df: self.df.clone(),
            output_port,
        }
    }

    pub fn merge<T>(&mut self) -> (InputPort<T>, Operator<T>)
    where
        T: Clone + 'static,
    {
        let mut df = (*self.df).borrow_mut();
        let (input, output_port) = df.add_op(move |recv, send| send.give_vec(&mut recv.take_all()));

        (
            input,
            Operator {
                df: self.df.clone(),
                output_port,
            },
        )
    }
}

#[test]
fn test_query() {
    let mut q = Query::new();

    q.source(|send| {
        send.push((1 as i64, "a".to_string()));
        send.push((2, "b".to_string()));
        send.push((3, "c".to_string()));
    })
    .join(q.source(|send| {
        send.push((1 as i64, "x".to_string()));
        send.push((2, "y".to_string()));
        send.push((2, "y2".to_string()));
        send.push((3, "z".to_string()));
    }))
    .sink(|i| println!("v: {:?}", i));

    (*q.df).borrow_mut().run();
}
