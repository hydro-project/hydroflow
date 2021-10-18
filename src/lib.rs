use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use slotmap::SlotMap;

type OpId = slotmap::DefaultKey;

pub trait Handoff<T> {
    type Readable: Readable<T>;
    type Writable: Writable<T>;
    type Meta: HandoffMeta;

    fn new() -> (Self::Readable, Self::Writable, Self::Meta);
}
pub trait Writable<T> {
    fn try_give(&mut self, item: T) -> Result<(), ()>;
}
pub trait Readable<T> {
    fn try_get(&mut self) -> Option<T>;
}
pub trait HandoffMeta {
    // TODO: more fine-grained info here.
    fn has_data(&self) -> bool;
}

impl<T> Handoff<T> for () {
    type Readable = ();
    type Writable = ();
    type Meta = ();

    fn new() -> (Self::Readable, Self::Writable, Self::Meta) {
        ((), (), ())
    }
}
impl<T> Writable<T> for () {
    fn try_give(&mut self, _item: T) -> Result<(), ()> {
        panic!("Tried to write to null handoff.");
    }
}
impl<T> Readable<T> for () {
    fn try_get(&mut self) -> Option<T> {
        panic!("Tried to read from null handoff.");
    }
}
impl HandoffMeta for () {
    fn has_data(&self) -> bool {
        false
    }
}

pub struct VecHandoff<T>(std::marker::PhantomData<T>);
impl<T> Handoff<T> for VecHandoff<T> {
    type Readable = Rc<RefCell<VecDeque<T>>>;
    type Writable = Rc<RefCell<VecDeque<T>>>;
    type Meta = Rc<RefCell<VecDeque<T>>>;

    fn new() -> (Self::Readable, Self::Writable, Self::Meta) {
        let v = Rc::new(RefCell::new(VecDeque::new()));
        (v.clone(), v.clone(), v)
    }
}
impl<T> Writable<T> for Rc<RefCell<VecDeque<T>>> {
    fn try_give(&mut self, t: T) -> Result<(), ()> {
        self.borrow_mut().push_back(t);
        Ok(())
    }
}
impl<T> Readable<T> for Rc<RefCell<VecDeque<T>>> {
    fn try_get(&mut self) -> Option<T> {
        self.borrow_mut().pop_front()
    }
}
impl<T> HandoffMeta for Rc<RefCell<VecDeque<T>>> {
    fn has_data(&self) -> bool {
        !self.borrow().is_empty()
    }
}

pub struct SendCtx<T, W: Writable<T>> {
    handoff: Rc<RefCell<Option<W>>>,

    _phantom: std::marker::PhantomData<T>,
}
impl<T, W: Writable<T>> SendCtx<T, W> {
    pub fn try_give(&self, item: T) -> Result<(), ()> {
        self.handoff.borrow_mut().as_mut().unwrap().try_give(item)
    }
}

pub struct OutputPort<T, W: Writable<T>> {
    handoff: Rc<RefCell<Option<W>>>,

    _phantom: std::marker::PhantomData<T>,
}

pub struct RecvCtx<T, R: Readable<T>> {
    handoff: Rc<RefCell<R>>,

    _phantom: std::marker::PhantomData<T>,
}
impl<T> Iterator for &RecvCtx<T, Rc<RefCell<VecDeque<T>>>> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODOTOTODOTOTODTOO !!!!!!!! TODO
        self.handoff.borrow_mut().borrow_mut().pop_front()
    }
}

// TODO: figure out how to explain succinctly why this and output port both use Writable
pub struct InputPort<T, W: Writable<T>> {
    handoff: W,

    _phantom: std::marker::PhantomData<T>,
}

pub trait OpSubtree {
    // TODO: pass in some scheduling info?
    fn run(&mut self);
}

struct ClosureOpSubtree<F, R, W, I, O>
where
    F: FnMut(&RecvCtx<I, R>, &SendCtx<O, W>),
    R: Readable<I>,
    W: Writable<O>,
{
    f: F,
    recv: RecvCtx<I, R>,
    send: SendCtx<O, W>,
    _phantom: std::marker::PhantomData<(I, O)>,
}
impl<F, R, W, I, O> OpSubtree for ClosureOpSubtree<F, R, W, I, O>
where
    F: FnMut(&RecvCtx<I, R>, &SendCtx<O, W>),
    R: Readable<I>,
    W: Writable<O>,
{
    fn run(&mut self) {
        (self.f)(&self.recv, &self.send)
    }
}

pub struct Dataflow {
    operators: SlotMap<OpId, (Box<dyn HandoffMeta>, Box<dyn OpSubtree>)>,
    // TODO: track the graph structure and schedule.
}
impl Dataflow {
    pub fn new() -> Self {
        Self {
            operators: Default::default(),
        }
    }
    pub fn run(&mut self) {
        for (_meta, op) in self.operators.values_mut() {
            op.run(); // TODO: TODOTOTODODOTOTOD
        }
        let mut any = true;
        while any {
            any = false;
            for (meta, op) in self.operators.values_mut() {
                if meta.has_data() {
                    op.run();
                    any = true;
                }
            }
        }
    }

    #[must_use]
    pub fn add_op<F, R, W, I, O>(
        &mut self,
        op_subtree: F,
    ) -> (InputPort<I, R::Writable>, OutputPort<O, W::Writable>)
    where
        F: 'static + FnMut(&RecvCtx<I, R::Readable>, &SendCtx<O, W::Writable>),
        R: 'static + Handoff<I>,
        W: 'static + Handoff<O>,
        I: 'static,
        O: 'static,
    {
        let (read_side, write_side, meta) = R::new(); // <-- !!

        // let send = self.make_send_ctx(id);
        // let s = send.clone();
        let recv = RecvCtx {
            handoff: Rc::new(RefCell::new(read_side)),

            _phantom: std::marker::PhantomData,
        };
        let send = SendCtx {
            handoff: Rc::new(RefCell::new(None)),

            _phantom: std::marker::PhantomData,
        };

        let input_port = InputPort {
            handoff: write_side,

            _phantom: std::marker::PhantomData,
        };
        let output_port = OutputPort {
            handoff: send.handoff.clone(),

            _phantom: std::marker::PhantomData,
        };

        let op: ClosureOpSubtree<F, R::Readable, W::Writable, I, O> = ClosureOpSubtree {
            f: op_subtree,
            recv,
            send,

            _phantom: std::marker::PhantomData,
        };

        self.operators.insert((Box::new(meta), Box::new(op)));

        (input_port, output_port)
    }

    #[must_use]
    pub fn add_source<F, W, O>(&mut self, mut op_subtree: F) -> OutputPort<O, W::Writable>
    where
        F: 'static + FnMut(&SendCtx<O, W::Writable>),
        W: 'static + Handoff<O>,
        O: 'static,
    {
        self.add_op::<_, (), W, (), O>(move |_, send| op_subtree(send))
            .1
    }

    #[must_use]
    pub fn add_sink<F, R, I>(&mut self, mut op_subtree: F) -> InputPort<I, R::Writable>
    where
        F: 'static + FnMut(&RecvCtx<I, R::Readable>),
        R: 'static + Handoff<I>,
        I: 'static,
    {
        self.add_op::<_, R, (), I, ()>(move |recv, _| op_subtree(recv))
            .0
    }

    pub fn add_edge<H, T>(
        &mut self,
        output_port: OutputPort<T, H::Writable>,
        input_port: InputPort<T, H::Writable>,
    ) where
        H: Handoff<T>,
    {
        let old_handoff = output_port.handoff.borrow_mut().replace(input_port.handoff);
        assert!(old_handoff.is_none());
    }
}

#[test]
fn map_filter() {
    let mut df = Dataflow::new();

    let mut data = vec![1, 2, 3, 4];
    let source = df.add_source::<_, VecHandoff<_>, _>(move |send| {
        for x in data.drain(..) {
            send.try_give(x).unwrap();
        }
    });

    let sink = df.add_sink::<_, VecHandoff<_>, _>(move |recv| {
        for x in recv {
            println!("x = {}", x);
        }
    });

    df.add_edge::<VecHandoff<_>, _>(source, sink);

    df.run();
}

// #[test]
// fn make_a_graph() {
//     let mut df = Dataflow::new();

// //    q
// //    ^ ___
// //    |/   \
// //    |     v
// //    r     f
// //   ^ ^   /
// //   |  \_/
// //   |
// //   s
// //

//     let q_i = df.add_sink(|recv| {
//         for r in recv {
//             println!("got data: {:?}", r):
//         }
//     });

//     // q here should be a WritableHandoff
//     // implementation of o
//     let (r_i1, r_i2, r_o1, r_o2) = df.add_op_2_2(|(i1, i2), (o1, o2)| {
//         for x in i1 {
//             o1.give(x.clone());
//             o2.give(x);
//         }
//         for x in i2 {
//             o1.give(x.clone());
//             o2.give(x);
//         }
//     });

//     // implementation of f
//     let (f_i, f_o) = df.add_op_1_1(|is, o| {
//         for i in is {
//             o.give(3*i + 1);
//         }
//     });

//     let s_o = df.add_source(|send| {
//         // something
//         send.send(5);
//         send.send(6);
//         send.send(7);
//     });

//     df.add_edge(r_o1, q_i);
//     df.add_edge(r_o2, f_i);
//     df.add_edge(f_o,  r_i2);
//     df.add_edge(s_o,  r_i1);
// }
