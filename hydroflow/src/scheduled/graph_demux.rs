use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

use super::{
    ctx::{InputPort, RecvCtx},
    graph::Hydroflow,
    graph_ext::GraphExt,
    handoff::{CanReceive, Handoff, VecHandoff},
    HandoffId, SubgraphId,
};

// This file implements a "demux" operation for Hydroflow graphs, to allow
// dynamically adding edges to a graph.

pub struct Demux<K, W> {
    outputs: Rc<RefCell<HashMap<K, usize>>>,
    id: SubgraphId,
    _marker: PhantomData<W>,
}

pub trait GraphDemux {
    /**
     * Add a [Demux] operator to the graph. Accepts a keying function to determine
     * how to dispatch any input values.
     */
    fn add_demux<F, T, K, W>(&mut self, sorter: F) -> (Demux<K, W>, InputPort<VecHandoff<T>>)
    where
        F: 'static + Fn(&T) -> K,
        K: 'static + std::hash::Hash + PartialEq + Eq,
        W: 'static + Handoff + CanReceive<Option<T>>,
        T: 'static + std::fmt::Debug;

    /**
     * Add another edge to demux, such that values who key to `k` will be sent
     * along `input_port`.
     */
    fn add_demux_edge<K, W>(&mut self, demux: &Demux<K, W>, k: K, input_port: InputPort<W>)
    where
        K: 'static + std::hash::Hash + PartialEq + Eq + std::fmt::Debug,
        W: 'static + Handoff;
}

impl GraphDemux for Hydroflow {
    // TODO(justin): don't hardcode the input handoff here?
    fn add_demux<F, T, K, W>(&mut self, sorter: F) -> (Demux<K, W>, InputPort<VecHandoff<T>>)
    where
        F: 'static + Fn(&T) -> K,
        K: 'static + std::hash::Hash + PartialEq + Eq,
        W: 'static + Handoff + CanReceive<Option<T>>,
        T: 'static + std::fmt::Debug,
    {
        let outputs_outer: Rc<RefCell<HashMap<K, HandoffId>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let outputs = outputs_outer.clone();
        let input = self.add_sink(move |ctx, recv: &RecvCtx<VecHandoff<T>>| {
            for v in recv.take_inner() {
                match (*outputs).borrow().get(&sorter(&v)) {
                    None => {
                        // TODO(justin): It's unclear what the right behaviour
                        // is if nobody has signed up for a given value. We could:
                        // 1. Drop them,
                        // 2. buffer them until someone comes along who wants them, or
                        // 3. send them to some default location.
                        // Probably we want this to be configurable because I
                        // can imagine sensible use-cases for all of these, but
                        // for now just go with (1).
                    }
                    Some(h_id) => {
                        let handoff = &ctx.handoffs[*h_id].handoff;
                        let maybe_handoff: Option<&W> = handoff.any_ref().downcast_ref();
                        let handoff = maybe_handoff.unwrap();
                        Handoff::give(handoff, Some(v));
                    }
                }
            }
        });
        (
            Demux {
                outputs: outputs_outer,
                id: input.sg_id,
                _marker: PhantomData,
            },
            input,
        )
    }

    fn add_demux_edge<K, W>(&mut self, demux: &Demux<K, W>, k: K, input_port: InputPort<W>)
    where
        K: 'static + std::hash::Hash + PartialEq + Eq + std::fmt::Debug,
        W: 'static + Handoff,
    {
        let handoff_id = self.add_handoff::<W>(demux.id, input_port.sg_id);

        input_port.handoff_id.set(Some(handoff_id));
        if (*demux.outputs)
            .borrow_mut()
            .insert(k, handoff_id)
            .is_some()
        {
            // TODO(justin): error here?
            panic!("demux key collision");
        }
    }
}
