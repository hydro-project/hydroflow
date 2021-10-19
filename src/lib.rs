use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use slotmap::SlotMap;

type OpId = slotmap::DefaultKey;

/**
 * A trait specifying a handoff point between compiled subgraphs.
 *
 * This trait is not meant to be instantiated directly, and instead provides a [Self::new()] associated function to create three separate pieces of a single handoff.
 */
pub trait Handoff {
    type Item;

    type Readable: Readable<Self::Item>;
    type Writable: Writable<Self::Item>;
    type Meta: HandoffMeta;

    fn new() -> (Self::Readable, Self::Writable, Self::Meta);
}
/**
 * The write piece of a handoff.
 */
pub trait Writable<T> {
    fn try_give(&mut self, item: T) -> Result<(), ()>;
}
/**
 * The read piece of a handoff.
 */
pub trait Readable<T> {
    fn try_get(&mut self) -> Option<T>;
}
/**
 * The metadata piece of a handoff.
 */
pub trait HandoffMeta {
    // TODO: more fine-grained info here.
    fn has_data(&self) -> bool;
}

/**
 * A null handoff which will panic when called.
 *
 * This is used in sources and sinks as the unused read or write handoff respectively.
 */
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

/**
 * A [VecDeque]-based FIFO handoff.
 */
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

/**
 * Context provided to a compiled component for writing to an [OutputPort].
 */
pub struct SendCtx<H: Handoff> {
    handoff: Rc<RefCell<Option<H::Writable>>>,
}
impl<H: Handoff> SendCtx<H> {
    // TODO: represent backpressure in this return value.
    pub fn try_give(&self, item: H::Item) -> Result<(), ()> {
        self.handoff.borrow_mut().as_mut().unwrap().try_give(item)
    }
}

/**
 * Handle corresponding to a [SendCtx]. Consumed by [Hydroflow::add_edge] to construct the Hydroflow graph.
 */
pub struct OutputPort<H: Handoff> {
    handoff: Rc<RefCell<Option<H::Writable>>>,
}

/**
 * Context provided to a compiled component for reading from an [InputPort].
 */
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

/**
 * Handle corresponding to a [RecvCtx]. Consumed by [Hydroflow::add_edge] to construct the Hydroflow graph.
 */
// TODO: figure out how to explain succinctly why this and output port both use Writable
pub struct InputPort<H: Handoff> {
    handoff: H::Writable,
}

/**
 * Represents a compiled subgraph. Used internally by [Dataflow] to erase the input/output [Handoff] types.
 */
trait Subgraph {
    // TODO: pass in some scheduling info?
    fn run(&mut self);
}

/**
 * Closure-based [OpSubtree] implementation.
 */
struct ClosureSubgraph<F, R, W>
where
    F: FnMut(&RecvCtx<R>, &SendCtx<W>),
    R: Handoff,
    W: Handoff,
{
    f: F,
    recv: RecvCtx<R>,
    send: SendCtx<W>,
}
impl<F, R, W> Subgraph for ClosureSubgraph<F, R, W>
where
    F: FnMut(&RecvCtx<R>, &SendCtx<W>),
    R: Handoff,
    W: Handoff,
{
    fn run(&mut self) {
        (self.f)(&self.recv, &self.send)
    }
}

/**
 * A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
 */
pub struct Hydroflow {
    subgraphs: SlotMap<OpId, (Box<dyn HandoffMeta>, Box<dyn Subgraph>)>,
    // TODO: track the graph structure and schedule.
}
impl Default for Hydroflow {
    fn default() -> Self {
        Hydroflow {
            subgraphs: Default::default(),
        }
    }
}
impl Hydroflow {
    /**
     * Create an new empty Dataflow graph.
     */
    pub fn new() -> Self {
        Default::default()
    }
    /**
     * Run the dataflow graph.
     *
     * Blocks until completion.
     */
    pub fn run(&mut self) {
        for (_meta, sg) in self.subgraphs.values_mut() {
            sg.run(); // TODO: TODOTOTODODOTOTOD
        }
        let mut any = true;
        while any {
            any = false;
            for (meta, sg) in self.subgraphs.values_mut() {
                if meta.has_data() {
                    sg.run();
                    any = true;
                }
            }
        }
    }

    /**
     * Adds a new compiled subraph with a single input and output, and returns the input/output handles.
     */
    #[must_use]
    pub fn add_inout<F, R, W>(&mut self, subgraph: F) -> (InputPort<R>, OutputPort<W>)
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

        let sg: ClosureSubgraph<F, R, W> = ClosureSubgraph {
            f: subgraph,
            recv,
            send,
        };

        self.subgraphs.insert((Box::new(meta), Box::new(sg)));

        (input_port, output_port)
    }

    /**
     * Adds a new compiled subgraph with no inputs and one output.
     */
    #[must_use]
    pub fn add_source<F, W>(&mut self, mut subgraph: F) -> OutputPort<W>
    where
        F: 'static + FnMut(&SendCtx<W>),
        W: 'static + Handoff,
    {
        self.add_inout::<_, NullHandoff, W>(move |_, send| subgraph(send))
            .1
    }

    /**
     * Adds a new compiled subgraph with one inputs and no outputs.
     */
    #[must_use]
    pub fn add_sink<F, R>(&mut self, mut subgraph: F) -> InputPort<R>
    where
        F: 'static + FnMut(&RecvCtx<R>),
        R: 'static + Handoff,
    {
        self.add_inout::<_, R, NullHandoff>(move |recv, _| subgraph(recv))
            .0
    }

    pub fn add_edge<H>(&mut self, output_port: OutputPort<H>, input_port: InputPort<H>)
    where
        H: Handoff,
    {
        let old_handoff = output_port.handoff.borrow_mut().replace(input_port.handoff);
        assert!(old_handoff.is_none());
    }
}

#[test]
fn map_filter() {
    // A simple dataflow with one source feeding into one sink.
    let mut df = Hydroflow::new();

    let data = [1, 2, 3, 4];
    let source = df.add_source(move |send| {
        for x in data.into_iter() {
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
