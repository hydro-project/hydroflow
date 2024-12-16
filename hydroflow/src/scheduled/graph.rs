//! Module for the [`Hydroflow`] struct and helper items.

use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::future::Future;
use std::marker::PhantomData;

use hydroflow_lang::diagnostic::{Diagnostic, SerdeSpan};
use hydroflow_lang::graph::HydroflowGraph;
use ref_cast::RefCast;
use smallvec::SmallVec;
use web_time::SystemTime;

use super::context::Context;
use super::handoff::handoff_list::PortList;
use super::handoff::{Handoff, HandoffMeta, TeeingHandoff};
use super::port::{RecvCtx, RecvPort, SendCtx, SendPort, RECV, SEND};
use super::reactor::Reactor;
use super::state::StateHandle;
use super::subgraph::Subgraph;
use super::{HandoffId, SubgraphId};
use crate::scheduled::ticks::{TickDuration, TickInstant};
use crate::Never;

/// A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
#[derive(Default)]
pub struct Hydroflow<'a> {
    pub(super) subgraphs: Vec<SubgraphData<'a>>,
    pub(super) context: Context,

    handoffs: Vec<HandoffData>,

    /// See [`Self::meta_graph()`].
    meta_graph: Option<HydroflowGraph>,
    /// See [`Self::diagnostics()`].
    diagnostics: Option<Vec<Diagnostic<SerdeSpan>>>,
}

/// Methods for [`TeeingHandoff`] teeing and dropping.
impl Hydroflow<'_> {
    /// Tees a [`TeeingHandoff`].
    pub fn teeing_handoff_tee<T>(
        &mut self,
        tee_parent_port: &RecvPort<TeeingHandoff<T>>,
    ) -> RecvPort<TeeingHandoff<T>>
    where
        T: Clone,
    {
        // Handoff ID of new tee output.
        let new_hoff_id = HandoffId(self.handoffs.len());

        // If we're teeing from a child make sure to find root.
        let tee_root = self.handoffs[tee_parent_port.handoff_id.0].pred_handoffs[0];

        // Set up teeing metadata.
        // Go to `tee_root`'s successors and insert self (the new tee output).
        let tee_root_data = &mut self.handoffs[tee_root.0];
        tee_root_data.succ_handoffs.push(new_hoff_id);

        // Add our new handoff id into the subgraph data if the send `tee_root` has already been
        // used to add a subgraph.
        assert!(
            tee_root_data.preds.len() <= 1,
            "Tee send side should only have one sender (or none set yet)."
        );
        if let Some(&pred_sg_id) = tee_root_data.preds.first() {
            self.subgraphs[pred_sg_id.0].succs.push(new_hoff_id);
        }

        // Insert new handoff output.
        let teeing_handoff = tee_root_data
            .handoff
            .any_ref()
            .downcast_ref::<TeeingHandoff<T>>()
            .unwrap();
        let new_handoff = teeing_handoff.tee();
        let new_name = Cow::Owned(format!("{} tee {:?}", tee_root_data.name, new_hoff_id));
        let mut new_handoff_data = HandoffData::new(new_name, new_handoff, new_hoff_id);
        // Set self's predecessor as `tee_root`.
        new_handoff_data.pred_handoffs = vec![tee_root];
        self.handoffs.push(new_handoff_data);

        let output_port = RecvPort {
            handoff_id: new_hoff_id,
            _marker: PhantomData,
        };
        output_port
    }

    /// Marks an output of a [`TeeingHandoff`] as dropped so that no more data will be sent to it.
    ///
    /// It is recommended to not not use this method and instead simply avoid teeing a
    /// [`TeeingHandoff`] when it is not needed.
    pub fn teeing_handoff_drop<T>(&mut self, tee_port: RecvPort<TeeingHandoff<T>>)
    where
        T: Clone,
    {
        let data = &self.handoffs[tee_port.handoff_id.0];
        let teeing_handoff = data
            .handoff
            .any_ref()
            .downcast_ref::<TeeingHandoff<T>>()
            .unwrap();
        teeing_handoff.drop();

        let tee_root = data.pred_handoffs[0];
        let tee_root_data = &mut self.handoffs[tee_root.0];
        // Remove this output from the send succ handoff list.
        tee_root_data
            .succ_handoffs
            .retain(|&succ_hoff| succ_hoff != tee_port.handoff_id);
        // Remove from subgraph successors if send port was already connected.
        assert!(
            tee_root_data.preds.len() <= 1,
            "Tee send side should only have one sender (or none set yet)."
        );
        if let Some(&pred_sg_id) = tee_root_data.preds.first() {
            self.subgraphs[pred_sg_id.0]
                .succs
                .retain(|&succ_hoff| succ_hoff != tee_port.handoff_id);
        }
    }
}

impl<'a> Hydroflow<'a> {
    /// Create a new empty Hydroflow graph.
    pub fn new() -> Self {
        Default::default()
    }

    /// Assign the `HydroflowGraph` via JSON string.
    #[doc(hidden)]
    pub fn __assign_meta_graph(&mut self, meta_graph_json: &str) {
        let mut meta_graph: HydroflowGraph =
            serde_json::from_str(meta_graph_json).expect("Failed to deserialize graph.");

        let mut op_inst_diagnostics = Vec::new();
        meta_graph.insert_node_op_insts_all(&mut op_inst_diagnostics);
        assert!(
            op_inst_diagnostics.is_empty(),
            "Expected no diagnostics, got: {:#?}",
            op_inst_diagnostics
        );

        assert!(self.meta_graph.replace(meta_graph).is_none());
    }
    /// Assign the diagnostics via JSON string.
    #[doc(hidden)]
    pub fn __assign_diagnostics(&mut self, diagnostics_json: &'static str) {
        let diagnostics: Vec<Diagnostic<SerdeSpan>> =
            serde_json::from_str(diagnostics_json).expect("Failed to deserialize diagnostics.");

        assert!(self.diagnostics.replace(diagnostics).is_none());
    }

    /// Return a handle to the meta `HydroflowGraph` if set. The `HydroflowGraph is a
    /// representation of all the operators, subgraphs, and handoffs in this `Hydroflow` instance.
    /// Will only be set if this graph was constructed using a surface syntax macro.
    pub fn meta_graph(&self) -> Option<&HydroflowGraph> {
        self.meta_graph.as_ref()
    }

    /// Returns any diagnostics generated by the surface syntax macro. Each diagnostic is a pair of
    /// (1) a `Diagnostic` with span info reset and (2) the `ToString` version of the diagnostic
    /// with original span info.
    /// Will only be set if this graph was constructed using a surface syntax macro.
    pub fn diagnostics(&self) -> Option<&[Diagnostic<SerdeSpan>]> {
        self.diagnostics.as_deref()
    }

    /// Returns a reactor for externally scheduling subgraphs, possibly from another thread.
    /// Reactor events are considered to be external events.
    pub fn reactor(&self) -> Reactor {
        Reactor::new(self.context.event_queue_send.clone())
    }

    /// Gets the current tick (local time) count.
    pub fn current_tick(&self) -> TickInstant {
        self.context.current_tick
    }

    /// Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.context.current_stratum
    }

    /// Runs the dataflow until the next tick begins.
    /// Returns true if any work was done.
    #[tracing::instrument(level = "trace", skip(self), ret)]
    pub fn run_tick(&mut self) -> bool {
        let mut work_done = false;
        // While work is immediately available *on the current tick*.
        while self.next_stratum(true) {
            work_done = true;
            // Do any work.
            self.run_stratum();
        }
        work_done
    }

    /// Runs the dataflow until no more (externally-triggered) work is immediately available.
    /// Runs at least one tick of dataflow, even if no external events have been received.
    /// If the dataflow contains loops this method may run forever.
    /// Returns true if any work was done.
    #[tracing::instrument(level = "trace", skip(self), ret)]
    pub fn run_available(&mut self) -> bool {
        let mut work_done = false;
        // While work is immediately available.
        while self.next_stratum(false) {
            work_done = true;
            // Do any work.
            self.run_stratum();
        }
        work_done
    }

    /// Runs the dataflow until no more (externally-triggered) work is immediately available.
    /// Runs at least one tick of dataflow, even if no external events have been received.
    /// If the dataflow contains loops this method may run forever.
    /// Returns true if any work was done.
    /// Yields repeatedly to allow external events to happen.
    #[tracing::instrument(level = "trace", skip(self), ret)]
    pub async fn run_available_async(&mut self) -> bool {
        let mut work_done = false;
        // While work is immediately available.
        while self.next_stratum(false) {
            work_done = true;
            // Do any work.
            self.run_stratum();

            // Yield between each stratum to receive more events.
            // TODO(mingwei): really only need to yield at start of ticks though.
            tokio::task::yield_now().await;
        }
        work_done
    }

    /// Runs the current stratum of the dataflow until no more local work is available (does not receive events).
    /// Returns true if any work was done.
    #[tracing::instrument(level = "trace", skip(self), fields(tick = u64::from(self.context.current_tick), stratum = self.context.current_stratum), ret)]
    pub fn run_stratum(&mut self) -> bool {
        // Make sure to spawn tasks once hydroflow is running!
        // This drains the task buffer, so becomes a no-op after first call.
        self.context.spawn_tasks();

        let current_tick = self.context.current_tick;

        let mut work_done = false;

        while let Some(sg_id) =
            self.context.stratum_queues[self.context.current_stratum].pop_front()
        {
            work_done = true;
            {
                let sg_data = &mut self.subgraphs[sg_id.0];
                // This must be true for the subgraph to be enqueued.
                assert!(sg_data.is_scheduled.take());
                tracing::trace!(
                    sg_id = sg_id.0,
                    sg_name = &*sg_data.name,
                    "Running subgraph."
                );

                self.context.subgraph_id = sg_id;
                self.context.subgraph_last_tick_run_in = sg_data.last_tick_run_in;
                sg_data.subgraph.run(&mut self.context, &mut self.handoffs);
                sg_data.last_tick_run_in = Some(current_tick);
            }

            let sg_data = &self.subgraphs[sg_id.0];
            for &handoff_id in sg_data.succs.iter() {
                let handoff = &self.handoffs[handoff_id.0];
                if !handoff.handoff.is_bottom() {
                    for &succ_id in handoff.succs.iter() {
                        let succ_sg_data = &self.subgraphs[succ_id.0];
                        // If we have sent data to the next tick, then we can start the next tick.
                        if succ_sg_data.stratum < self.context.current_stratum && !sg_data.is_lazy {
                            self.context.can_start_tick = true;
                        }
                        // Add subgraph to stratum queue if it is not already scheduled.
                        if !succ_sg_data.is_scheduled.replace(true) {
                            self.context.stratum_queues[succ_sg_data.stratum].push_back(succ_id);
                        }
                    }
                }
            }

            for sg_id in self.context.rescheduled_subgraphs.borrow_mut().drain(..) {
                let sg_data = &self.subgraphs[sg_id.0];
                assert_eq!(sg_data.stratum, self.context.current_stratum);
                if !sg_data.is_scheduled.replace(true) {
                    self.context.stratum_queues[sg_data.stratum].push_back(sg_id);
                }
            }
        }
        work_done
    }

    /// Go to the next stratum which has work available, possibly the current stratum.
    /// Return true if more work is available, otherwise false if no work is immediately
    /// available on any strata.
    ///
    /// This will receive external events when at the start of a tick.
    ///
    /// If `current_tick_only` is set to `true`, will only return `true` if work is immediately
    /// available on the *current tick*.
    ///
    /// If this returns false then the graph will be at the start of a tick (at stratum 0, can
    /// receive more external events).
    #[tracing::instrument(level = "trace", skip(self), ret)]
    pub fn next_stratum(&mut self, current_tick_only: bool) -> bool {
        tracing::trace!(
            events_received_tick = self.context.events_received_tick,
            can_start_tick = self.context.can_start_tick,
            "Starting `next_stratum` call.",
        );

        if 0 == self.context.current_stratum {
            // Starting the tick, reset this to `false`.
            tracing::trace!("Starting tick, setting `can_start_tick = false`.");
            self.context.can_start_tick = false;
            self.context.current_tick_start = SystemTime::now();

            // Ensure external events are received before running the tick.
            if !self.context.events_received_tick {
                // Add any external jobs to ready queue.
                self.try_recv_events();
            }
        }

        // The stratum we will stop searching at, i.e. made a full loop around.
        let mut end_stratum = self.context.current_stratum;

        loop {
            tracing::trace!(
                tick = u64::from(self.context.current_tick),
                stratum = self.context.current_stratum,
                "Looking for work on stratum."
            );

            // If current stratum has work, return true.
            if !self.context.stratum_queues[self.context.current_stratum].is_empty() {
                tracing::trace!(
                    tick = u64::from(self.context.current_tick),
                    stratum = self.context.current_stratum,
                    "Work found on stratum."
                );
                return true;
            }

            // Increment stratum counter.
            self.context.current_stratum += 1;
            if self.context.current_stratum >= self.context.stratum_queues.len() {
                tracing::trace!(
                    can_start_tick = self.context.can_start_tick,
                    "End of tick {}, starting tick {}.",
                    self.context.current_tick,
                    self.context.current_tick + TickDuration::SINGLE_TICK,
                );
                self.context.reset_state_at_end_of_tick();

                self.context.current_stratum = 0;
                self.context.current_tick += TickDuration::SINGLE_TICK;
                self.context.events_received_tick = false;

                if current_tick_only {
                    tracing::trace!(
                        "`current_tick_only` is `true`, returning `false` before receiving events."
                    );
                    return false;
                } else {
                    self.try_recv_events();
                    if std::mem::replace(&mut self.context.can_start_tick, false) {
                        tracing::trace!(
                            tick = u64::from(self.context.current_tick),
                            "`can_start_tick` is `true`, continuing."
                        );
                        // Do a full loop more to find where events have been added.
                        end_stratum = 0;
                        continue;
                    } else {
                        tracing::trace!(
                            "`can_start_tick` is `false`, re-setting `events_received_tick = false`, returning `false`."
                        );
                        self.context.events_received_tick = false;
                        return false;
                    }
                }
            }

            // After incrementing, exit if we made a full loop around the strata.
            if end_stratum == self.context.current_stratum {
                tracing::trace!("Made full loop around stratum, re-setting `current_stratum = 0`, returning `false`.");
                // Note: if current stratum had work, the very first loop iteration would've
                // returned true. Therefore we can return false without checking.
                // Also means nothing was done so we can reset the stratum to zero and wait for
                // events.
                self.context.events_received_tick = false;
                self.context.current_stratum = 0;
                return false;
            }
        }
    }

    /// Runs the dataflow graph forever.
    ///
    /// TODO(mingwei): Currently blocks forever, no notion of "completion."
    #[tracing::instrument(level = "trace", skip(self), ret)]
    pub fn run(&mut self) -> Option<Never> {
        loop {
            self.run_tick();
        }
    }

    /// Runs the dataflow graph forever.
    ///
    /// TODO(mingwei): Currently blocks forever, no notion of "completion."
    #[tracing::instrument(level = "trace", skip(self), ret)]
    pub async fn run_async(&mut self) -> Option<Never> {
        loop {
            // Run any work which is immediately available.
            self.run_available_async().await;
            // When no work is available yield until more events occur.
            self.recv_events_async().await;
        }
    }

    /// Enqueues subgraphs triggered by events without blocking.
    ///
    /// Returns the number of subgraphs enqueued, and if any were external.
    #[tracing::instrument(level = "trace", skip(self), fields(events_received_tick = self.context.events_received_tick), ret)]
    pub fn try_recv_events(&mut self) -> usize {
        let mut enqueued_count = 0;
        while let Ok((sg_id, is_external)) = self.context.event_queue_recv.try_recv() {
            let sg_data = &self.subgraphs[sg_id.0];
            tracing::trace!(
                sg_id = sg_id.0,
                is_external = is_external,
                sg_stratum = sg_data.stratum,
                "Event received."
            );
            if !sg_data.is_scheduled.replace(true) {
                self.context.stratum_queues[sg_data.stratum].push_back(sg_id);
                enqueued_count += 1;
            }
            if is_external {
                // Next tick is triggered if we are at the start of the next tick (`!self.events_receved_tick`).
                // Or if the stratum is in the next tick.
                if !self.context.events_received_tick
                    || sg_data.stratum < self.context.current_stratum
                {
                    tracing::trace!(
                        current_stratum = self.context.current_stratum,
                        sg_stratum = sg_data.stratum,
                        "External event, setting `can_start_tick = true`."
                    );
                    self.context.can_start_tick = true;
                }
            }
        }
        self.context.events_received_tick = true;

        enqueued_count
    }

    /// Enqueues subgraphs triggered by external events, blocking until at
    /// least one subgraph is scheduled **from an external event**.
    #[tracing::instrument(level = "trace", skip(self), fields(events_received_tick = self.context.events_received_tick), ret)]
    pub fn recv_events(&mut self) -> Option<usize> {
        let mut count = 0;
        loop {
            let (sg_id, is_external) = self.context.event_queue_recv.blocking_recv()?;
            let sg_data = &self.subgraphs[sg_id.0];
            tracing::trace!(
                sg_id = sg_id.0,
                is_external = is_external,
                sg_stratum = sg_data.stratum,
                "Event received."
            );
            if !sg_data.is_scheduled.replace(true) {
                self.context.stratum_queues[sg_data.stratum].push_back(sg_id);
                count += 1;
            }
            if is_external {
                // Next tick is triggered if we are at the start of the next tick (`!self.events_receved_tick`).
                // Or if the stratum is in the next tick.
                if !self.context.events_received_tick
                    || sg_data.stratum < self.context.current_stratum
                {
                    tracing::trace!(
                        current_stratum = self.context.current_stratum,
                        sg_stratum = sg_data.stratum,
                        "External event, setting `can_start_tick = true`."
                    );
                    self.context.can_start_tick = true;
                }
                break;
            }
        }
        self.context.events_received_tick = true;

        // Enqueue any other immediate events.
        let extra_count = self.try_recv_events();
        Some(count + extra_count)
    }

    /// Enqueues subgraphs triggered by external events asynchronously, waiting until at least one
    /// subgraph is scheduled **from an external event**. Returns the number of subgraphs enqueued,
    /// which may be zero if an external event scheduled an already-scheduled subgraph.
    ///
    /// Returns `None` if the event queue is closed, but that should not happen normally.
    #[tracing::instrument(level = "trace", skip(self), fields(events_received_tick = self.context.events_received_tick), ret)]
    pub async fn recv_events_async(&mut self) -> Option<usize> {
        let mut count = 0;
        loop {
            tracing::trace!("Awaiting events (`event_queue_recv`).");
            let (sg_id, is_external) = self.context.event_queue_recv.recv().await?;
            let sg_data = &self.subgraphs[sg_id.0];
            tracing::trace!(
                sg_id = sg_id.0,
                is_external = is_external,
                sg_stratum = sg_data.stratum,
                "Event received."
            );
            if !sg_data.is_scheduled.replace(true) {
                self.context.stratum_queues[sg_data.stratum].push_back(sg_id);
                count += 1;
            }
            if is_external {
                // Next tick is triggered if we are at the start of the next tick (`!self.events_receved_tick`).
                // Or if the stratum is in the next tick.
                if !self.context.events_received_tick
                    || sg_data.stratum < self.context.current_stratum
                {
                    tracing::trace!(
                        current_stratum = self.context.current_stratum,
                        sg_stratum = sg_data.stratum,
                        "External event, setting `can_start_tick = true`."
                    );
                    self.context.can_start_tick = true;
                }
                break;
            }
        }
        self.context.events_received_tick = true;

        // Enqueue any other immediate events.
        let extra_count = self.try_recv_events();
        Some(count + extra_count)
    }

    /// Schedules a subgraph to be run. See also: [`Context::schedule_subgraph`].
    pub fn schedule_subgraph(&mut self, sg_id: SubgraphId) -> bool {
        let sg_data = &self.subgraphs[sg_id.0];
        let already_scheduled = sg_data.is_scheduled.replace(true);
        if !already_scheduled {
            self.context.stratum_queues[sg_data.stratum].push_back(sg_id);
            true
        } else {
            false
        }
    }

    /// Adds a new compiled subgraph with the specified inputs and outputs in stratum 0.
    pub fn add_subgraph<Name, R, W, F>(
        &mut self,
        name: Name,
        recv_ports: R,
        send_ports: W,
        subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'static + for<'ctx> FnMut(&'ctx mut Context, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        self.add_subgraph_stratified(name, 0, recv_ports, send_ports, false, subgraph)
    }

    /// Adds a new compiled subgraph with the specified inputs, outputs, and stratum number.
    ///
    /// TODO(mingwei): add example in doc.
    pub fn add_subgraph_stratified<Name, R, W, F>(
        &mut self,
        name: Name,
        stratum: usize,
        recv_ports: R,
        send_ports: W,
        laziness: bool,
        mut subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'a + for<'ctx> FnMut(&'ctx mut Context, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        let sg_id = SubgraphId(self.subgraphs.len());

        let (mut subgraph_preds, mut subgraph_succs) = Default::default();
        recv_ports.set_graph_meta(&mut self.handoffs, &mut subgraph_preds, sg_id, true);
        send_ports.set_graph_meta(&mut self.handoffs, &mut subgraph_succs, sg_id, false);

        let subgraph = move |context: &mut Context, handoffs: &mut Vec<HandoffData>| {
            let recv = recv_ports.make_ctx(&*handoffs);
            let send = send_ports.make_ctx(&*handoffs);
            (subgraph)(context, recv, send);
        };
        self.subgraphs.push(SubgraphData::new(
            name.into(),
            stratum,
            subgraph,
            subgraph_preds,
            subgraph_succs,
            true,
            laziness,
        ));
        self.context.init_stratum(stratum);
        self.context.stratum_queues[stratum].push_back(sg_id);

        sg_id
    }

    /// Adds a new compiled subgraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_n_m<Name, R, W, F>(
        &mut self,
        name: Name,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static
            + for<'ctx> FnMut(&'ctx mut Context, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
    {
        self.add_subgraph_stratified_n_m(name, 0, recv_ports, send_ports, subgraph)
    }

    /// Adds a new compiled subgraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_stratified_n_m<Name, R, W, F>(
        &mut self,
        name: Name,
        stratum: usize,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        mut subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static
            + for<'ctx> FnMut(&'ctx mut Context, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
    {
        let sg_id = SubgraphId(self.subgraphs.len());

        let subgraph_preds = recv_ports.iter().map(|port| port.handoff_id).collect();
        let subgraph_succs = send_ports.iter().map(|port| port.handoff_id).collect();

        for recv_port in recv_ports.iter() {
            self.handoffs[recv_port.handoff_id.0].succs.push(sg_id);
        }
        for send_port in send_ports.iter() {
            self.handoffs[send_port.handoff_id.0].preds.push(sg_id);
        }

        let subgraph = move |context: &mut Context, handoffs: &mut Vec<HandoffData>| {
            let recvs: Vec<&RecvCtx<R>> = recv_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| handoffs.get(hid.0).unwrap())
                .map(|h_data| {
                    h_data
                        .handoff
                        .any_ref()
                        .downcast_ref()
                        .expect("Attempted to cast handoff to wrong type.")
                })
                .map(RefCast::ref_cast)
                .collect();

            let sends: Vec<&SendCtx<W>> = send_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| handoffs.get(hid.0).unwrap())
                .map(|h_data| {
                    h_data
                        .handoff
                        .any_ref()
                        .downcast_ref()
                        .expect("Attempted to cast handoff to wrong type.")
                })
                .map(RefCast::ref_cast)
                .collect();

            (subgraph)(context, &recvs, &sends)
        };
        self.subgraphs.push(SubgraphData::new(
            name.into(),
            stratum,
            subgraph,
            subgraph_preds,
            subgraph_succs,
            true,
            false,
        ));
        self.context.init_stratum(stratum);
        self.context.stratum_queues[stratum].push_back(sg_id);

        sg_id
    }

    /// Creates a handoff edge and returns the corresponding send and receive ports.
    pub fn make_edge<Name, H>(&mut self, name: Name) -> (SendPort<H>, RecvPort<H>)
    where
        Name: Into<Cow<'static, str>>,
        H: 'static + Handoff,
    {
        let handoff_id = HandoffId(self.handoffs.len());

        // Create and insert handoff.
        let handoff = H::default();
        self.handoffs
            .push(HandoffData::new(name.into(), handoff, handoff_id));

        // Make ports.
        let input_port = SendPort {
            handoff_id,
            _marker: PhantomData,
        };
        let output_port = RecvPort {
            handoff_id,
            _marker: PhantomData,
        };
        (input_port, output_port)
    }

    /// Adds referenceable state into the `Hydroflow` instance. Returns a state handle which can be
    /// used externally or by operators to access the state.
    ///
    /// This is part of the "state API".
    pub fn add_state<T>(&mut self, state: T) -> StateHandle<T>
    where
        T: Any,
    {
        self.context.add_state(state)
    }

    /// Sets a hook to modify the state at the end of each tick, using the supplied closure.
    ///
    /// This is part of the "state API".
    pub fn set_state_tick_hook<T>(
        &mut self,
        handle: StateHandle<T>,
        tick_hook_fn: impl 'static + FnMut(&mut T),
    ) where
        T: Any,
    {
        self.context.set_state_tick_hook(handle, tick_hook_fn)
    }

    /// Gets a exclusive (mut) ref to the internal context, setting the subgraph ID.
    pub fn context_mut(&mut self, sg_id: SubgraphId) -> &mut Context {
        self.context.subgraph_id = sg_id;
        &mut self.context
    }
}

impl Hydroflow<'_> {
    /// Alias for [`Context::request_task`].
    pub fn request_task<Fut>(&mut self, future: Fut)
    where
        Fut: Future<Output = ()> + 'static,
    {
        self.context.request_task(future);
    }

    /// Alias for [`Context::abort_tasks`].
    pub fn abort_tasks(&mut self) {
        self.context.abort_tasks()
    }

    /// Alias for [`Context::join_tasks`].
    pub fn join_tasks(&mut self) -> impl '_ + Future {
        self.context.join_tasks()
    }
}

impl Drop for Hydroflow<'_> {
    fn drop(&mut self) {
        self.abort_tasks();
    }
}

/// A handoff and its input and output [SubgraphId]s.
///
/// Internal use: used to track the hydroflow graph structure.
///
/// TODO(mingwei): restructure `PortList` so this can be crate-private.
#[doc(hidden)]
pub struct HandoffData {
    /// A friendly name for diagnostics.
    pub(super) name: Cow<'static, str>,
    /// Crate-visible to crate for `handoff_list` internals.
    pub(super) handoff: Box<dyn HandoffMeta>,
    /// Preceeding subgraphs (including the send side of a teeing handoff).
    pub(super) preds: SmallVec<[SubgraphId; 1]>,
    /// Successor subgraphs (including recv sides of teeing handoffs).
    pub(super) succs: SmallVec<[SubgraphId; 1]>,

    /// Predecessor handoffs, used by teeing handoffs.
    /// Should be `self` on any teeing send sides (input).
    /// Should be the send `HandoffId` if this is teeing recv side (output).
    /// Should be just `self`'s `HandoffId` on other handoffs.
    /// This field is only used in initialization.
    pub(super) pred_handoffs: Vec<HandoffId>,
    /// Successor handoffs, used by teeing handoffs.
    /// Should be a list of outputs on the teeing send side (input).
    /// Should be `self` on any teeing recv sides (outputs).
    /// Should be just `self`'s `HandoffId` on other handoffs.
    /// This field is only used in initialization.
    pub(super) succ_handoffs: Vec<HandoffId>,
}
impl std::fmt::Debug for HandoffData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("HandoffData")
            .field("preds", &self.preds)
            .field("succs", &self.succs)
            .finish_non_exhaustive()
    }
}
impl HandoffData {
    /// New with `pred_handoffs` and `succ_handoffs` set to its own [`HandoffId`]: `vec![hoff_id]`.
    pub fn new(
        name: Cow<'static, str>,
        handoff: impl 'static + HandoffMeta,
        hoff_id: HandoffId,
    ) -> Self {
        let (preds, succs) = Default::default();
        Self {
            name,
            handoff: Box::new(handoff),
            preds,
            succs,
            pred_handoffs: vec![hoff_id],
            succ_handoffs: vec![hoff_id],
        }
    }
}

/// A subgraph along with its predecessor and successor [SubgraphId]s.
///
/// Used internally by the [Hydroflow] struct to represent the dataflow graph
/// structure and scheduled state.
pub(super) struct SubgraphData<'a> {
    /// A friendly name for diagnostics.
    pub(super) name: Cow<'static, str>,
    /// This subgraph's stratum number.
    pub(super) stratum: usize,
    /// The actual execution code of the subgraph.
    subgraph: Box<dyn Subgraph + 'a>,

    #[expect(dead_code, reason = "may be useful in the future")]
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,

    /// If this subgraph is scheduled in [`Hydroflow::stratum_queues`].
    /// [`Cell`] allows modifying this field when iterating `Self::preds` or
    /// `Self::succs`, as all `SubgraphData` are owned by the same vec
    /// `Hydroflow::subgraphs`.
    is_scheduled: Cell<bool>,

    /// Keep track of the last tick that this subgraph was run in
    last_tick_run_in: Option<TickInstant>,

    /// If this subgraph is marked as lazy, then sending data back to a lower stratum does not trigger a new tick to be run.
    is_lazy: bool,
}
impl<'a> SubgraphData<'a> {
    pub fn new(
        name: Cow<'static, str>,
        stratum: usize,
        subgraph: impl Subgraph + 'a,
        preds: Vec<HandoffId>,
        succs: Vec<HandoffId>,
        is_scheduled: bool,
        laziness: bool,
    ) -> Self {
        Self {
            name,
            stratum,
            subgraph: Box::new(subgraph),
            preds,
            succs,
            is_scheduled: Cell::new(is_scheduled),
            last_tick_run_in: None,
            is_lazy: laziness,
        }
    }
}
