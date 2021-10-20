use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    rc::Rc,
};

mod query;

pub use query::{Operator, Query};

// TODO: make this work without clone.
#[derive(Default, Debug, Clone)]
struct Schedule<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    order: VecDeque<T>,
    members: HashSet<T>,
}

impl<T> Schedule<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn new() -> Self {
        Schedule {
            order: VecDeque::new(),
            members: HashSet::new(),
        }
    }

    fn insert(&mut self, t: T) {
        if !self.members.contains(&t) {
            self.members.insert(t.clone());
            self.order.push_back(t)
        }
    }

    fn pop(&mut self) -> Option<T> {
        let v = self.order.pop_front()?;
        self.members.remove(&v);
        Some(v)
    }
}

#[derive(Default)]
pub struct Dataflow {
    // TODO: transpose these.
    operators: Vec<Box<dyn FnMut()>>,
    dirties: Vec<Vec<Rc<RefCell<bool>>>>,
    schedule: Rc<RefCell<Schedule<usize>>>,
    adjacencies: Vec<Vec<usize>>,
}

pub struct RecvCtx<T> {
    inputs: Rc<RefCell<Vec<T>>>,
}

impl<T> RecvCtx<T> {
    fn new(inputs: Rc<RefCell<Vec<T>>>) -> Self {
        RecvCtx { inputs }
    }
}

impl<I> RecvCtx<I> {
    pub fn pull(&self) -> Option<I> {
        (*self.inputs).borrow_mut().pop()
    }

    pub fn take_all(&self) -> Vec<I> {
        (*self.inputs).replace(Vec::new())
    }
}

#[derive(Clone)]
pub struct SendCtx<O>
where
    O: Clone,
{
    id: usize,
    #[allow(clippy::type_complexity)]
    subscribers: Rc<RefCell<Vec<Rc<RefCell<Vec<O>>>>>>,
    dirty: Rc<RefCell<bool>>,
}

impl<O> SendCtx<O>
where
    O: Clone,
{
    pub fn push(&self, o: O) {
        for sub in &*(*self.subscribers).borrow() {
            (*sub).borrow_mut().push(o.clone())
        }
        *(*self.dirty).borrow_mut() = true;
    }

    pub fn give_vec(&self, v: &mut Vec<O>) {
        let subs = &*(*self.subscribers).borrow_mut();
        for sub in subs.iter().take(subs.len() - 1) {
            sub.borrow_mut().extend_from_slice(v);
        }
        if !subs.is_empty() {
            if (*subs[subs.len() - 1]).borrow().len() == 0 {
                (*subs[subs.len() - 1]).replace(std::mem::take(v));
            } else {
                (*subs[subs.len() - 1]).borrow_mut().extend(v.drain(..));
            }
        }
        *(*self.dirty).borrow_mut() = true;
    }

    pub fn give_iterator<I>(&self, v: I)
    where
        I: IntoIterator<Item = O>,
    {
        let subs = &*(*self.subscribers).borrow_mut();
        if subs.is_empty() {
            return;
        }
        let mut first = (*subs[0]).borrow_mut();
        let l = (*first).len();
        first.extend(v);
        // Now copy that data from the first one over to the rest.
        for sub in subs.iter().skip(1) {
            (*sub).borrow_mut().extend_from_slice(&first[l..]);
        }
        *(*self.dirty).borrow_mut() = true;
    }
}

#[derive(Clone)]
pub struct InputPort<T> {
    id: usize,
    data: MessageBuffer<T>,
}

#[derive(Debug, Clone)]
struct MessageBuffer<T> {
    data: Rc<RefCell<Vec<T>>>,
}

impl<T> MessageBuffer<T> {
    fn new() -> (Self, RecvCtx<T>) {
        let data = Rc::new(RefCell::new(Vec::new()));
        let d2 = data.clone();
        (MessageBuffer { data }, RecvCtx::new(d2))
    }
}

impl Dataflow {
    pub fn new() -> Self {
        Dataflow {
            operators: Vec::new(),
            dirties: Vec::new(),
            adjacencies: Vec::new(),
            schedule: Rc::new(RefCell::new(Schedule::new())),
        }
    }

    pub fn run(&mut self) {
        while let Some(id) = (*self.schedule).borrow_mut().pop() {
            self.operators[id]();

            // If that operator sent out any data, its corresponding dirty bit will be true, so
            // we can schedule all of its downstream operators.
            if *(*self.dirties[id][0]).borrow() {
                *(*self.dirties[id][0]).borrow_mut() = false;
                for op in &self.adjacencies[id] {
                    (*self.schedule).borrow_mut().insert(*op);
                }
            }
        }
    }

    pub fn add_edge<T: Clone>(&mut self, o: SendCtx<T>, i: InputPort<T>) {
        (*o.subscribers).borrow_mut().push(i.data.data.clone());
        self.adjacencies[o.id].push(i.id);
    }

    pub fn add_source<F: 'static, O: 'static>(&mut self, mut f: F) -> SendCtx<O>
    where
        F: FnMut(&SendCtx<O>),
        O: Clone,
    {
        self.add_op(move |_recv: &RecvCtx<()>, send| f(send)).1
    }

    pub fn add_sink<F: 'static, I: 'static>(&mut self, mut f: F) -> InputPort<I>
    where
        F: FnMut(&RecvCtx<I>),
        I: Clone,
    {
        self.add_op(move |recv, _send: &SendCtx<()>| f(recv)).0
    }

    fn make_send_ctx<T>(&mut self, id: usize) -> SendCtx<T>
    where
        T: Clone,
    {
        SendCtx {
            id,
            subscribers: Rc::new(RefCell::new(Vec::new())),
            dirty: Rc::new(RefCell::new(false)),
        }
    }

    pub fn add_op_2<F: 'static, I1: 'static, I2: 'static, O: 'static>(
        &mut self,
        mut f: F,
    ) -> (InputPort<I1>, InputPort<I2>, SendCtx<O>)
    where
        F: FnMut(&RecvCtx<I1>, &RecvCtx<I2>, &SendCtx<O>),
        O: Clone,
    {
        let id = self.operators.len();
        let (buf1, recv1) = MessageBuffer::new();
        let (buf2, recv2) = MessageBuffer::new();

        let send = self.make_send_ctx(id);
        let s = send.clone();
        let op = move || f(&recv1, &recv2, &s);

        self.operators.push(Box::new(op));
        self.dirties.push(vec![send.dirty.clone()]);
        self.adjacencies.push(Vec::new());
        (*self.schedule).borrow_mut().insert(id);

        (
            InputPort { id, data: buf1 },
            InputPort { id, data: buf2 },
            send,
        )
    }

    pub fn add_op<F: 'static, I: 'static, O: 'static>(
        &mut self,
        mut f: F,
    ) -> (InputPort<I>, SendCtx<O>)
    where
        F: FnMut(&RecvCtx<I>, &SendCtx<O>),
        O: Clone,
    {
        let id = self.operators.len();
        let (inputs, recv) = MessageBuffer::new();

        let send = self.make_send_ctx(id);
        let s = send.clone();
        let op = move || f(&recv, &s);

        self.operators.push(Box::new(op));
        self.dirties.push(vec![send.dirty.clone()]);
        self.adjacencies.push(Vec::new());
        (*self.schedule).borrow_mut().insert(id);

        (InputPort { id, data: inputs }, send)
    }
}

#[test]
fn test_df() {
    let mut df = Dataflow::new();

    let mut sent = false;

    let output = df.add_source(move |ctx| {
        if !sent {
            sent = true;
            ctx.push(1);
            ctx.push(2);
            ctx.push(3);
        }
    });

    let input = df.add_sink(|ctx| {
        while let Some(v) = ctx.pull() {
            println!("v = {}", v);
        }
    });

    df.add_edge(output, input);

    df.run();
}

#[test]
fn test_df_binary() {
    let mut df = Dataflow::new();

    let mut sent = false;

    let source1 = df.add_source(move |ctx| {
        if !sent {
            sent = true;
            ctx.push(1);
            ctx.push(2);
            ctx.push(3);
        }
    });

    let source2 = df.add_source(move |ctx| {
        if !sent {
            sent = true;
            ctx.push(2);
            ctx.push(3);
            ctx.push(4);
        }
    });

    let (input1, input2, _) = df.add_op_2(|r1, r2, _: &SendCtx<()>| {
        while let Some(v) = r1.pull() {
            println!("left = {}", v);
        }
        while let Some(v) = r2.pull() {
            println!("right = {}", v);
        }
    });

    df.add_edge(source1, input1);
    df.add_edge(source2, input2);

    df.run();
}
