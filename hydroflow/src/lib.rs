#![feature(never_type)]

pub mod collections;
pub mod handoff;
pub mod util;

use std::cell::RefCell;
use std::rc::Rc;

use sealed::sealed;
use slotmap::SlotMap;
pub use tuple_list::tuple_list as tl;

use handoff::Handoff;
use handoff::HandoffMeta;
use handoff::NullHandoff;
use handoff::VecHandoff;
use handoff::{CanReceive, TryCanReceive};

pub type OpId = slotmap::DefaultKey;

/**
 * Context provided to a compiled component for writing to an [OutputPort].
 */
pub struct SendCtx<H: Handoff> {
    once: util::Once<Rc<RefCell<H>>>,
}
impl<H: Handoff> SendCtx<H> {
    // // TODO: represent backpressure in this return value.
    // #[allow(clippy::result_unit_err)]
    // pub fn give(self, item: H::Item) -> Result<(), ()> {
    //     (*self.once.get()).borrow_mut().try_give(item)
    // }
    pub fn give<T>(&mut self, item: T) -> T
    where
        H: CanReceive<T>,
    {
        let mut borrow = self.once.get().borrow_mut();
        <H as CanReceive<T>>::give(&mut *borrow, item)
    }

    pub fn try_give<T>(&mut self, item: T) -> Result<T, T>
    where
        H: TryCanReceive<T>,
    {
        let mut borrow = self.once.get().borrow_mut();
        <H as TryCanReceive<T>>::try_give(&mut *borrow, item)
    }
}

/**
 * Handle corresponding to a [SendCtx]. Consumed by [Hydroflow::add_edge] to construct the Hydroflow graph.
 */
#[must_use]
pub struct OutputPort<H: Handoff> {
    once: util::SendOnce<Rc<RefCell<H>>>,
}

/**
 * Context provided to a compiled component for reading from an [InputPort].
 */
pub struct RecvCtx<H: Handoff> {
    handoff: Rc<RefCell<H>>,
}
impl<T> IntoIterator for &RecvCtx<VecHandoff<T>> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut borrow = self.handoff.borrow_mut();
        let vec = std::mem::take(&mut *borrow);
        vec.0.into_iter()
    }
}

// impl<T> Iterator for &RecvCtx<VecHandoff<T>> {
//     type Item = T;

//     fn next(&mut self) -> Option<Self::Item> {
//         // TODOTOTODOTOTODTOO !!!!!!!! TODO
//         self.handoff.borrow_mut().0.pop_front()
//     }
// }

/**
 * Handle corresponding to a [RecvCtx]. Consumed by [Hydroflow::add_edge] to construct the Hydroflow graph.
 */
// TODO: figure out how to explain succinctly why this and output port both use Writable
#[must_use]
pub struct InputPort<H: Handoff> {
    handoff: Rc<RefCell<H>>,
}

/**
 * A variadic list of Handoff types, represented using a lisp-style tuple structure.
 *
 * This trait is sealed and not meant to be implemented or used directly. Instead tuple lists (which already implement this trait) should be used, for example:
 * ```ignore
 * type MyHandoffList = (VecHandoff<usize>, (VecHandoff<String>, (NullHandoff, ())));
 * ```
 * The [`tl!`] (tuple list) macro simplifies usage of this kind:
 * ```ignore
 * type MyHandoffList = tl!(VecHandoff<usize>, VecHandoff<String>, NullHandoff);
 * ```
 */
#[sealed]
pub trait HandoffList {
    type RecvCtx;
    type InputPort;
    type Meta;
    fn make_input() -> (Self::RecvCtx, Self::InputPort, Self::Meta);

    type SendCtx;
    type OutputPort;
    fn make_output() -> (Self::SendCtx, Self::OutputPort);

    fn append_meta(vec: &mut Vec<Box<dyn HandoffMeta>>, meta: Self::Meta);
}
#[sealed]
impl<H, L> HandoffList for (H, L)
where
    H: 'static + Handoff,
    L: HandoffList,
{
    type RecvCtx = (RecvCtx<H>, L::RecvCtx);
    type InputPort = (InputPort<H>, L::InputPort);
    type Meta = (Rc<RefCell<H>>, L::Meta);
    fn make_input() -> (Self::RecvCtx, Self::InputPort, Self::Meta) {
        let handoff = H::default();

        let handoff = Rc::new(RefCell::new(handoff));

        let recv = RecvCtx {
            handoff: handoff.clone(),
        };
        let input = InputPort {
            handoff: handoff.clone(),
        };

        let (recv_rest, input_rest, meta_rest) = L::make_input();

        ((recv, recv_rest), (input, input_rest), (handoff, meta_rest))
    }

    type SendCtx = (SendCtx<H>, L::SendCtx);
    type OutputPort = (OutputPort<H>, L::OutputPort);
    fn make_output() -> (Self::SendCtx, Self::OutputPort) {
        let (once_send, once_recv) = util::once();

        let send = SendCtx { once: once_recv };
        let output = OutputPort { once: once_send };

        let (send_rest, output_rest) = L::make_output();

        ((send, send_rest), (output, output_rest))
    }

    fn append_meta(vec: &mut Vec<Box<dyn HandoffMeta>>, meta: Self::Meta) {
        let (meta, meta_rest) = meta;
        vec.push(Box::new(meta));
        L::append_meta(vec, meta_rest);
    }
}
#[sealed]
impl HandoffList for () {
    type RecvCtx = ();
    type InputPort = ();
    type Meta = ();
    fn make_input() -> (Self::RecvCtx, Self::InputPort, Self::Meta) {
        ((), (), ())
    }

    type SendCtx = ();
    type OutputPort = ();
    fn make_output() -> (Self::SendCtx, Self::OutputPort) {
        ((), ())
    }

    fn append_meta(_vec: &mut Vec<Box<dyn HandoffMeta>>, _meta: Self::Meta) {}
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
struct VariadicClosureSubgraph<F, R, W>
where
    F: FnMut(&mut R::RecvCtx, &mut W::SendCtx),
    R: HandoffList,
    W: HandoffList,
{
    f: F,
    recv: R::RecvCtx,
    send: W::SendCtx,
}
impl<F, R, W> Subgraph for VariadicClosureSubgraph<F, R, W>
where
    F: FnMut(&mut R::RecvCtx, &mut W::SendCtx),
    R: HandoffList,
    W: HandoffList,
{
    fn run(&mut self) {
        (self.f)(&mut self.recv, &mut self.send)
    }
}

/**
 * A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
 */
pub struct Hydroflow {
    // TODO(justin): instead of this being a vec of metas, it could be
    // implemented for 2-tuples of metas.
    #[allow(clippy::type_complexity)]
    subgraphs: SlotMap<OpId, (Vec<Box<dyn HandoffMeta>>, Box<dyn Subgraph>)>,
    // TODO: track the graph structure and schedule.
}
#[allow(clippy::derivable_impls)]
impl Default for Hydroflow {
    fn default() -> Self {
        Self {
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
            for (metas, sg) in self.subgraphs.values_mut() {
                if metas.iter().any(|m| !m.is_bottom()) {
                    sg.run();
                    any = true;
                }
            }
        }
    }

    /**
     * Adds a new compiled subgraph with the specified inputs and outputs.
     *
     * See [HandoffList] for how to specify inputs and outputs.
     */
    #[must_use]
    pub fn add_subgraph<F, R, W>(&mut self, subgraph: F) -> (R::InputPort, W::OutputPort)
    where
        F: 'static + FnMut(&mut R::RecvCtx, &mut W::SendCtx),
        R: 'static + HandoffList,
        W: 'static + HandoffList,
    {
        let (recv, input_port, meta) = R::make_input();
        let (send, output_port) = W::make_output();

        let sg: VariadicClosureSubgraph<F, R, W> = VariadicClosureSubgraph {
            f: subgraph,
            recv,
            send,
        };

        let mut meta_vec = Vec::new();
        R::append_meta(&mut meta_vec, meta);

        self.subgraphs.insert((meta_vec, Box::new(sg)));

        (input_port, output_port)
    }

    /**
     * Adds a new compiled subraph with a single input and output, and returns the input/output handles.
     */
    pub fn add_inout<F, R, W>(&mut self, mut subgraph: F) -> (InputPort<R>, OutputPort<W>)
    where
        F: 'static + FnMut(&mut RecvCtx<R>, &mut SendCtx<W>),
        R: 'static + Handoff,
        W: 'static + Handoff,
    {
        let (tl!(input_port), tl!(output_port)) = self
            .add_subgraph::<_, tl!(R), tl!(W)>(move |tl!(recv), tl!(send)| (subgraph)(recv, send));
        (input_port, output_port)
    }

    /**
     * Adds a new compiled subraph with one input and two outputs, and returns the input/output handles.
     */
    pub fn add_binary_out<F, R, W1, W2>(
        &mut self,
        mut subgraph: F,
    ) -> (InputPort<R>, OutputPort<W1>, OutputPort<W2>)
    where
        F: 'static + FnMut(&mut RecvCtx<R>, &mut SendCtx<W1>, &mut SendCtx<W2>),
        R: 'static + Handoff,
        W1: 'static + Handoff,
        W2: 'static + Handoff,
    {
        let (tl!(input_port), tl!(output_port1, output_port2)) = self
            .add_subgraph::<_, tl!(R), tl!(W1, W2)>(move |tl!(recv), tl!(send1, send2)| {
                (subgraph)(recv, send1, send2)
            });
        (input_port, output_port1, output_port2)
    }

    /**
     * Adds a new compiled subraph with two inputs and a single output, and returns the input/output handles.
     */
    pub fn add_binary<F, R1, R2, W>(
        &mut self,
        mut subgraph: F,
    ) -> (InputPort<R1>, InputPort<R2>, OutputPort<W>)
    where
        F: 'static + FnMut(&mut RecvCtx<R1>, &mut RecvCtx<R2>, &mut SendCtx<W>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
        W: 'static + Handoff,
    {
        let (tl!(input_port1, input_port2), tl!(output_port)) = self
            .add_subgraph::<_, tl!(R1, R2), tl!(W)>(move |tl!(recv1, recv2), tl!(send)| {
                (subgraph)(recv1, recv2, send)
            });
        (input_port1, input_port2, output_port)
    }

    /**
     * Adds a new compiled subgraph with no inputs and one output.
     */
    pub fn add_source<F, W>(&mut self, mut subgraph: F) -> OutputPort<W>
    where
        F: 'static + FnMut(&mut SendCtx<W>),
        W: 'static + Handoff,
    {
        self.add_inout::<_, NullHandoff, W>(move |_, send| subgraph(send))
            .1
    }

    /**
     * Adds a new compiled subgraph with one inputs and no outputs.
     */
    pub fn add_sink<F, R>(&mut self, mut subgraph: F) -> InputPort<R>
    where
        F: 'static + FnMut(&mut RecvCtx<R>),
        R: 'static + Handoff,
    {
        self.add_inout::<_, R, NullHandoff>(move |recv, _| subgraph(recv))
            .0
    }

    pub fn add_edge<H>(&mut self, output_port: OutputPort<H>, input_port: InputPort<H>)
    where
        H: Handoff,
    {
        output_port.once.send(input_port.handoff);
    }
}

#[test]
fn map_filter() {
    // A simple dataflow with one source feeding into one sink with some processing in the middle.
    let mut df = Hydroflow::new();

    let data = [1, 2, 3, 4];
    let source = df.add_source(move |send| {
        for x in data.into_iter() {
            send.give(Some(x));
        }
    });

    let (map_in, map_out) = df.add_inout(
        |recv: &mut RecvCtx<VecHandoff<i32>>, send: &mut SendCtx<VecHandoff<_>>| {
            for x in &*recv {
                send.give(Some(3 * x + 1));
            }
        },
    );

    let (filter_in, filter_out) = df.add_inout(
        |recv: &mut RecvCtx<VecHandoff<i32>>, send: &mut SendCtx<VecHandoff<_>>| {
            for x in &*recv {
                if x % 2 == 0 {
                    send.give(Some(x));
                }
            }
        },
    );

    let outputs = Rc::new(RefCell::new(Vec::new()));
    let inner_outputs = outputs.clone();
    let sink = df.add_sink(move |recv| {
        for x in &*recv {
            (*inner_outputs).borrow_mut().push(x);
        }
    });

    df.add_edge(source, map_in);
    df.add_edge(map_out, filter_in);
    df.add_edge(filter_out, sink);

    df.run();

    assert_eq!((*outputs).borrow().clone(), vec![4, 10]);
}

mod tests {
    #![allow(unused_imports)]
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        rc::Rc,
    };

    use crate::{Hydroflow, RecvCtx, SendCtx, VecHandoff};

    #[test]
    fn test_cycle() {
        // A dataflow that represents graph reachability.

        let mut edges: HashMap<usize, Vec<usize>> = HashMap::new();
        for (from, to) in &[
            (1_usize, 2_usize),
            (1, 3),
            (1, 4),
            (2, 3),
            (2, 5),
            (5, 1),
            (6, 7),
            (7, 8),
        ] {
            edges.entry(*from).or_insert_with(Vec::new).push(*to);
        }

        let mut df = Hydroflow::new();

        let mut initially_reachable = vec![1];
        let reachable = df.add_source(move |send: &mut SendCtx<VecHandoff<usize>>| {
            for v in initially_reachable.drain(..) {
                send.give(Some(v));
            }
        });

        let mut seen = HashSet::new();
        let (distinct_in, distinct_out) = df.add_inout(
            move |recv: &mut RecvCtx<VecHandoff<usize>>, send: &mut SendCtx<VecHandoff<usize>>| {
                for v in &*recv {
                    if seen.insert(v) {
                        send.give(Some(v));
                    }
                }
            },
        );

        let (merge_lhs, merge_rhs, merge_out) =
            df.add_binary(|recv1, recv2, send: &mut SendCtx<VecHandoff<usize>>| {
                for v in (recv1.into_iter()).chain(recv2.into_iter()) {
                    send.give(Some(v));
                }
            });

        let (neighbors_in, neighbors_out) = df.add_inout(move |recv, send| {
            for v in &*recv {
                if let Some(neighbors) = edges.get(&v) {
                    for &n in neighbors {
                        send.give(Some(n));
                    }
                }
            }
        });

        let (tee_in, tee_out1, tee_out2) = df.add_binary_out(
            |recv: &mut RecvCtx<VecHandoff<usize>>,
             send1: &mut SendCtx<VecHandoff<usize>>,
             send2: &mut SendCtx<VecHandoff<usize>>| {
                for v in &*recv {
                    send1.give(Some(v));
                    send2.give(Some(v));
                }
            },
        );

        let reachable_verts = Rc::new(RefCell::new(Vec::new()));
        let reachable_inner = reachable_verts.clone();
        let sink_in = df.add_sink(move |recv| {
            for v in &*recv {
                (*reachable_inner).borrow_mut().push(v);
            }
        });

        df.add_edge(reachable, merge_lhs);
        df.add_edge(neighbors_out, merge_rhs);
        df.add_edge(merge_out, distinct_in);
        df.add_edge(distinct_out, tee_in);
        df.add_edge(tee_out1, neighbors_in);
        df.add_edge(tee_out2, sink_in);

        df.run();

        assert_eq!((*reachable_verts).borrow().clone(), vec![1, 2, 3, 4, 5]);
    }
}
