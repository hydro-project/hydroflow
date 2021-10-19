use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use slotmap::SlotMap;

type OpId = slotmap::DefaultKey;

pub trait Handoff {
    type Item;

    type Readable: Readable<Self::Item>;
    type Writable: Writable<Self::Item>;
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

pub enum NullHandoff {}
impl Handoff for NullHandoff {
    type Item = ();

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
impl<T> Handoff for VecHandoff<T> {
    type Item = T;

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

pub struct SendCtx<H: Handoff> {
    handoff: Rc<RefCell<Option<H::Writable>>>,
}
impl<H: Handoff> SendCtx<H> {
    pub fn try_give(&self, item: H::Item) -> Result<(), ()> {
        self.handoff.borrow_mut().as_mut().unwrap().try_give(item)
    }
}

pub struct OutputPort<H: Handoff> {
    handoff: Rc<RefCell<Option<H::Writable>>>,
}

pub struct RecvCtx<H: Handoff> {
    handoff: Rc<RefCell<H::Readable>>,
}
impl<T> Iterator for &RecvCtx<VecHandoff<T>> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODOTOTODOTOTODTOO !!!!!!!! TODO
        self.handoff.borrow_mut().borrow_mut().pop_front()
    }
}

// TODO: figure out how to explain succinctly why this and output port both use Writable
pub struct InputPort<H: Handoff> {
    handoff: H::Writable,
}

pub trait OpSubtree {
    // TODO: pass in some scheduling info?
    fn run(&mut self);
}

struct ClosureOpSubtree<F, R, W>
where
    F: FnMut(&RecvCtx<R>, &SendCtx<W>),
    R: Handoff,
    W: Handoff,
{
    f: F,
    recv: RecvCtx<R>,
    send: SendCtx<W>,
}
impl<F, R, W> OpSubtree for ClosureOpSubtree<F, R, W>
where
    F: FnMut(&RecvCtx<R>, &SendCtx<W>),
    R: Handoff,
    W: Handoff,
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
    pub fn add_op<F, R, W>(
        &mut self,
        op_subtree: F,
    ) -> (InputPort<R>, OutputPort<W>)
    where
        F: 'static + FnMut(&RecvCtx<R>, &SendCtx<W>),
        R: 'static + Handoff,
        W: 'static + Handoff,
    {
        let (read_side, write_side, meta) = R::new();

        let recv = RecvCtx {
            handoff: Rc::new(RefCell::new(read_side)),
        };
        let send = SendCtx {
            handoff: Rc::new(RefCell::new(None)),
        };

        let input_port = InputPort {
            handoff: write_side,
        };
        let output_port = OutputPort {
            handoff: send.handoff.clone(),
        };

        let op: ClosureOpSubtree<F, R, W> = ClosureOpSubtree {
            f: op_subtree,
            recv,
            send,
        };

        self.operators.insert((Box::new(meta), Box::new(op)));

        (input_port, output_port)
    }

    #[must_use]
    pub fn add_source<F, W>(&mut self, mut op_subtree: F) -> OutputPort<W>
    where
        F: 'static + FnMut(&SendCtx<W>),
        W: 'static + Handoff,
    {
        self.add_op::<_, NullHandoff, W>(move |_, send| op_subtree(send))
            .1
    }

    #[must_use]
    pub fn add_sink<F, R>(&mut self, mut op_subtree: F) -> InputPort<R>
    where
        F: 'static + FnMut(&RecvCtx<R>),
        R: 'static + Handoff,
    {
        self.add_op::<_, R, NullHandoff>(move |recv, _| op_subtree(recv))
            .0
    }

    pub fn add_edge<H>(
        &mut self,
        output_port: OutputPort<H>,
        input_port: InputPort<H>,
    ) where
        H: Handoff,
    {
        let old_handoff = output_port.handoff.borrow_mut().replace(input_port.handoff);
        assert!(old_handoff.is_none());
    }
}

#[test]
fn map_filter() {
    let mut df = Dataflow::new();

    let data = [1, 2, 3, 4];
    let source = df.add_source(move |send| {
        for x in data {
            send.try_give(x).unwrap();
        }
    });

    let sink = df.add_sink(move |recv| {
        for x in recv {
            println!("x = {}", x);
        }
    });

    df.add_edge(source, sink);

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
