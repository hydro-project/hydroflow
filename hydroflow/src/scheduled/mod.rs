pub mod collections;
pub mod ctx;
pub mod handoff;
pub mod query;
pub(crate) mod subgraph;
pub mod util;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender, TrySendError};

use crate::tl;
use ctx::{InputPort, OutputPort, RecvCtx, SendCtx};
use handoff::{Handoff, HandoffList, HandoffMeta, NullHandoff, VecHandoff};
use subgraph::{NtoMClosureSubgraph, Subgraph, VariadicClosureSubgraph};

pub type OpId = usize;
pub type HandoffId = usize;

/**
 * A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
 */
pub struct Hydroflow {
    subgraphs: Vec<SubgraphData>,
    handoffs: Vec<HandoffData>,

    // TODO(mingwei): separate scheduler into its own struct/trait?
    ready_queue: VecDeque<OpId>,
    event_queue_send: SyncSender<OpId>, // TODO(mingwei) remove this, to prevent hanging.
    event_queue_recv: Receiver<OpId>,
}
impl Default for Hydroflow {
    fn default() -> Self {
        let (subgraphs, handoffs, ready_queue) = Default::default();
        let (event_queue_send, event_queue_recv) = mpsc::sync_channel(8_000);
        Self {
            subgraphs,
            handoffs,
            ready_queue,
            event_queue_send,
            event_queue_recv,
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
     * Returns a reactor for externally scheduling operators, possibly from another thread.
     */
    pub fn reactor(&self) -> Reactor {
        Reactor {
            event_queue_send: self.event_queue_send.clone(),
        }
    }

    /**
     * Runs the dataflow until no more work is currently available.
     */
    pub fn tick(&mut self) {
        loop {
            // Add any external jobs to ready queue.
            self.ready_queue.extend(self.event_queue_recv.try_iter());

            if let Some(op_id) = self.ready_queue.pop_front() {
                let sg_data = self.subgraphs.get_mut(op_id).unwrap();
                sg_data.subgraph.run();
                for handoff_id in sg_data.succs.iter().copied() {
                    let handoff = self.handoffs.get(handoff_id).unwrap();
                    let succ_id = handoff.succ;
                    if self.ready_queue.contains(&succ_id) {
                        // TODO(mingwei): Slow? O(N)
                        continue;
                    }
                    if !handoff.handoff.is_bottom() {
                        self.ready_queue.push_back(succ_id);
                    }
                }
            } else {
                break;
            }
        }
    }

    /**
     * Run the dataflow graph, blocking until completion.
     */
    pub fn run(&mut self) -> Result<!, RecvError> {
        loop {
            // Do any current work.
            self.tick();

            // Block and wait for an external event.
            self.ready_queue.push_back(self.event_queue_recv.recv()?);
        }
    }

    /**
     * TODO(mingwei): Hack to re-enqueue all subgraphs.
     */
    pub fn wake_all(&mut self) {
        self.ready_queue.clear();
        self.ready_queue.extend(0..self.subgraphs.len());
    }

    /**
     * Adds a new compiled subgraph with the specified inputs and outputs.
     *
     * See [HandoffList] for how to specify inputs and outputs.
     */
    #[must_use]
    pub fn add_subgraph<F, R, W>(&mut self, f: F) -> (R::InputPort, W::OutputPort)
    where
        F: 'static + FnMut(&mut R::RecvCtx, &mut W::SendCtx),
        R: 'static + HandoffList,
        W: 'static + HandoffList,
    {
        // TODO(justin): make this less sketchy, we just know we're the only person who will append here.
        let op_id = self.subgraphs.len();

        let (recv, input_port) = R::make_input(op_id);
        let (send, output_port) = W::make_output(op_id);

        let subgraph = VariadicClosureSubgraph::<F, R, W>::new(f, recv, send);
        self.subgraphs.push(SubgraphData::new(subgraph));
        self.ready_queue.push_back(op_id);

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
     * Adds a new compiled subraph with a variable number of inputs and outputs.
     */
    pub fn add_n_in_m_out<F, R, W>(
        &mut self,
        n: usize,
        m: usize,
        f: F,
    ) -> (Vec<InputPort<R>>, Vec<OutputPort<W>>)
    where
        F: 'static + FnMut(&mut [RecvCtx<R>], &mut [SendCtx<W>]),
        R: 'static + Handoff,
        W: 'static + Handoff,
    {
        let mut recvs = Vec::new();
        let mut input_ports = Vec::new();
        let mut input_metas = Vec::new();
        let op_id = self.subgraphs.len();

        for _ in 0..n {
            let handoff = Rc::new(RefCell::new(R::default()));
            let once = Rc::new(RefCell::new(None));
            recvs.push(RecvCtx { once: once.clone() });
            input_ports.push(InputPort { once, op_id });
            input_metas.push(Box::new(handoff) as Box<dyn HandoffMeta>);
        }

        let mut sends = Vec::new();
        let mut output_ports = Vec::new();

        for _ in 0..m {
            let handoff = Rc::new(RefCell::new(W::default()));

            sends.push(SendCtx {
                handoff: handoff.clone(),
            });
            output_ports.push(OutputPort { op_id, handoff });
        }

        let subgraph = NtoMClosureSubgraph::<F, R, W>::new(f, recvs, sends);
        self.subgraphs.push(SubgraphData::new(subgraph));
        self.ready_queue.push_back(op_id);

        (input_ports, output_ports)
    }

    /**
     * Adds a new compiled subraph with one input and two outputs, and returns the input/output handles.
     */
    pub fn add_binary_in_binary_out<F, R1, R2, W1, W2>(
        &mut self,
        mut subgraph: F,
    ) -> (InputPort<R1>, InputPort<R2>, OutputPort<W1>, OutputPort<W2>)
    where
        F: 'static + FnMut(&mut RecvCtx<R1>, &mut RecvCtx<R2>, &mut SendCtx<W1>, &mut SendCtx<W2>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
        W1: 'static + Handoff,
        W2: 'static + Handoff,
    {
        let (tl!(input_port1, input_port2), tl!(output_port1, output_port2)) = self
            .add_subgraph::<_, tl!(R1, R2), tl!(W1, W2)>(
                move |tl!(recv1, recv2), tl!(send1, send2)| (subgraph)(recv1, recv2, send1, send2),
            );
        (input_port1, input_port2, output_port1, output_port2)
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
     * Adds a new compiled subraph with two inputs and no outputs, and returns the input handles.
     */
    pub fn add_binary_sink<F, R1, R2>(&mut self, mut subgraph: F) -> (InputPort<R1>, InputPort<R2>)
    where
        F: 'static + FnMut(&mut RecvCtx<R1>, &mut RecvCtx<R2>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
    {
        let (tl!(input_port1, input_port2), tl!()) =
            self.add_subgraph::<_, tl!(R1, R2), tl!()>(move |tl!(recv1, recv2), tl!()| {
                (subgraph)(recv1, recv2)
            });
        (input_port1, input_port2)
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
        H: 'static + Handoff,
    {
        // Insert handoff.
        let handoff_id: HandoffId = self.handoffs.len();
        self.handoffs.push(HandoffData::new(
            output_port.handoff.clone(),
            output_port.op_id,
            input_port.op_id,
        ));

        // Add successor.
        self.subgraphs[output_port.op_id].succs.push(handoff_id);
        // Add predacessor.
        self.subgraphs[input_port.op_id].preds.push(handoff_id);

        *input_port.once.borrow_mut() = Some(output_port.handoff);
    }
}

/**
 * A handle into a specific [Hydroflow] instance for triggering operators to run, possibly from another thread.Default
 */
#[derive(Clone)]
pub struct Reactor {
    event_queue_send: SyncSender<OpId>,
}
impl Reactor {
    pub fn trigger(&self, op_id: OpId) -> Result<(), TrySendError<usize>> {
        self.event_queue_send.try_send(op_id)
    }

    #[cfg(feature = "async")]
    pub fn into_waker(self, op_id: OpId) -> std::task::Waker {
        use futures::task::ArcWake;
        use std::sync::Arc;

        struct ReactorWaker {
            reactor: Reactor,
            op_id: OpId,
        }
        impl ArcWake for ReactorWaker {
            fn wake_by_ref(arc_self: &Arc<Self>) {
                arc_self.reactor.trigger(arc_self.op_id).unwrap(/* TODO(mingwei) */);
            }
        }

        let reactor_waker = ReactorWaker {
            reactor: self,
            op_id,
        };
        futures::task::waker(Arc::new(reactor_waker))
    }
}

/**
 * A handoff and its input and output [OpId]s.
 */
struct HandoffData {
    handoff: Box<dyn HandoffMeta>,
    #[allow(dead_code)] // TODO(mingwei)
    pred: OpId,
    succ: OpId,
}
impl HandoffData {
    pub fn new(handoff: impl 'static + HandoffMeta, pred: OpId, succ: OpId) -> Self {
        Self {
            handoff: Box::new(handoff),
            pred,
            succ,
        }
    }
}

/**
 * A subgraph along with its predacessor and successor [OpId]s.
 * Used internally by the [Hydroflow] struct to represent the dataflow graph structure.
 */
struct SubgraphData {
    subgraph: Box<dyn Subgraph>,
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,
}
impl SubgraphData {
    pub fn new(subgraph: impl 'static + Subgraph) -> Self {
        Self {
            subgraph: Box::new(subgraph),
            preds: Default::default(),
            succs: Default::default(),
        }
    }
}

#[test]
fn map_filter() {
    use std::cell::RefCell;
    use std::rc::Rc;

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
            for x in recv.take_inner().into_iter() {
                send.give(Some(3 * x + 1));
            }
        },
    );

    let (filter_in, filter_out) = df.add_inout(
        |recv: &mut RecvCtx<VecHandoff<i32>>, send: &mut SendCtx<VecHandoff<_>>| {
            for x in recv.take_inner().into_iter() {
                if x % 2 == 0 {
                    send.give(Some(x));
                }
            }
        },
    );

    let outputs = Rc::new(RefCell::new(Vec::new()));
    let inner_outputs = outputs.clone();
    let sink = df.add_sink(move |recv: &mut RecvCtx<VecHandoff<i32>>| {
        for x in recv.take_inner().into_iter() {
            (*inner_outputs).borrow_mut().push(x);
        }
    });

    df.add_edge(source, map_in);
    df.add_edge(map_out, filter_in);
    df.add_edge(filter_out, sink);

    df.tick();

    assert_eq!((*outputs).borrow().clone(), vec![4, 10]);
}

mod tests {
    #![allow(unused_imports)]
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        rc::Rc,
    };

    use crate::scheduled::{handoff::Handoff, Hydroflow, RecvCtx, SendCtx, VecHandoff};

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
                for v in recv.take_inner().into_iter() {
                    if seen.insert(v) {
                        send.give(Some(v));
                    }
                }
            },
        );

        let (merge_lhs, merge_rhs, merge_out) = df.add_binary(
            |recv1: &mut RecvCtx<VecHandoff<usize>>,
             recv2: &mut RecvCtx<VecHandoff<usize>>,
             send: &mut SendCtx<VecHandoff<usize>>| {
                for v in (recv1.take_inner().into_iter()).chain(recv2.take_inner().into_iter()) {
                    send.give(Some(v));
                }
            },
        );

        let (neighbors_in, neighbors_out) =
            df.add_inout(move |recv: &mut RecvCtx<VecHandoff<usize>>, send| {
                for v in recv.take_inner().into_iter() {
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
                for v in recv.take_inner().into_iter() {
                    send1.give(Some(v));
                    send2.give(Some(v));
                }
            },
        );

        let reachable_verts = Rc::new(RefCell::new(Vec::new()));
        let reachable_inner = reachable_verts.clone();
        let sink_in = df.add_sink(move |recv: &mut RecvCtx<VecHandoff<usize>>| {
            for v in recv.take_inner().into_iter() {
                (*reachable_inner).borrow_mut().push(v);
            }
        });

        df.add_edge(reachable, merge_lhs);
        df.add_edge(neighbors_out, merge_rhs);
        df.add_edge(merge_out, distinct_in);
        df.add_edge(distinct_out, tee_in);
        df.add_edge(tee_out1, neighbors_in);
        df.add_edge(tee_out2, sink_in);

        df.tick();

        assert_eq!((*reachable_verts).borrow().clone(), vec![1, 2, 3, 4, 5]);
    }
}

#[test]
fn test_auto_tee() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::scheduled::handoff::TeeingHandoff;

    let mut df = Hydroflow::new();

    let mut data = vec![1, 2, 3, 4];
    let source = df.add_source(move |send: &mut SendCtx<TeeingHandoff<_>>| {
        send.give(std::mem::take(&mut data));
    });

    let out1 = Rc::new(RefCell::new(Vec::new()));
    let out1_inner = out1.clone();

    let sink1 = df.add_sink(move |recv: &mut RecvCtx<_>| {
        for v in recv.take_inner() {
            out1_inner.borrow_mut().extend(v);
        }
    });

    let out2 = Rc::new(RefCell::new(Vec::new()));
    let out2_inner = out2.clone();
    let sink2 = df.add_sink(move |recv: &mut RecvCtx<_>| {
        for v in recv.take_inner() {
            out2_inner.borrow_mut().extend(v);
        }
    });

    df.add_edge(source.clone(), sink1);
    df.add_edge(source, sink2);

    df.tick();

    assert_eq!((*out1).borrow().clone(), vec![1, 2, 3, 4]);
    assert_eq!((*out2).borrow().clone(), vec![1, 2, 3, 4]);
}
