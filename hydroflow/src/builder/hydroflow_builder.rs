use super::build::{PullBuild, PushBuild};
use super::connect::{PullConnect, PushConnect};
use super::surface::pivot::PivotSurface;

use std::rc::Rc;
use std::sync::mpsc::SyncSender;

use crate::compiled::pivot::Pivot;
use crate::scheduled::graph::Hydroflow;
use crate::scheduled::graph_ext::GraphExt;
use crate::scheduled::handoff::{CanReceive, Handoff, VecHandoff};
use crate::scheduled::input::Input;
use crate::scheduled::net::Message;

use super::surface::pull_handoff::HandoffPullSurface;
use super::surface::push_handoff::HandoffPushSurfaceReversed;
use super::surface::push_start::StartPushSurface;
use super::surface::{PullSurface, PushSurfaceReversed};

/// The user-facing entry point for the Surface API.
#[derive(Default)]
pub struct HydroflowBuilder {
    hydroflow: Hydroflow,
    // TODO(mingwei): use a dedicated trait instead of FnOnce?
    handoff_connectors: Vec<Box<dyn FnOnce(&mut Hydroflow)>>,
}
impl HydroflowBuilder {
    /// Creates a handoff, returning push and pull ends which can be chained
    /// using the Surface API.
    pub fn make_handoff<H, T>(
        &mut self,
    ) -> (HandoffPushSurfaceReversed<H, T>, HandoffPullSurface<H>)
    where
        H: Handoff + CanReceive<T>,
    {
        let push_port = Default::default();
        let push = HandoffPushSurfaceReversed::new(Rc::clone(&push_port));

        let pull_port = Default::default();
        let pull = HandoffPullSurface::new(Rc::clone(&pull_port));

        self.handoff_connectors.push(Box::new(move |hydroflow| {
            if let (Some(output_port), Some(input_port)) = (push_port.take(), pull_port.take()) {
                hydroflow.add_edge(output_port, input_port);
            } else {
                panic!("Handoff was never connected!!"); // TODO(mingwei): more informative error messages.
            }
        }));
        (push, pull)
    }

    /// Adds a `pivot` created via the Surface API.
    pub fn add_subgraph<Pull, Push>(&mut self, pivot: PivotSurface<Pull, Push>)
    where
        Pull: 'static + PullSurface,
        Push: 'static + PushSurfaceReversed<ItemIn = Pull::ItemOut>,
    {
        let ((pull_connect, push_connect), (mut pull_build, mut push_build)) = pivot.into_parts();

        let (input_ports, output_ports) = self
            .hydroflow
            .add_subgraph::<_, Pull::InputHandoffs, Push::OutputHandoffs>(
                move |_ctx, recv_ctx, send_ctx| {
                    let pull = pull_build.build(recv_ctx);
                    let push = push_build.build(send_ctx);
                    let pivot = Pivot::new(pull, push);
                    pivot.run();
                },
            );

        pull_connect.connect(input_ports);
        push_connect.connect(output_ports);
    }

    /// Creates a new external channel input.
    pub fn add_channel_input<T, W>(&mut self) -> (Input<T, SyncSender<T>>, HandoffPullSurface<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>,
    {
        let (input, output_port) = self.hydroflow.add_channel_input();

        let pull_port = Default::default();
        let pull = HandoffPullSurface::new(Rc::clone(&pull_port));

        self.handoff_connectors.push(Box::new(move |hydroflow| {
            if let Some(input_port) = pull_port.take() {
                hydroflow.add_edge(output_port, input_port);
            } else {
                panic!("Channel input was never connected!!"); // TODO(mingwei): more informative error messages.
            }
        }));

        (input, pull)
    }

    pub fn add_write_tcp_stream(&mut self, stream: tokio::net::TcpStream) -> HandoffPushSurfaceReversed<VecHandoff<Message>, Option<Message>> {
        let input_port = self.hydroflow.add_write_tcp_stream(stream);

        let push_port = Default::default();
        let push = HandoffPushSurfaceReversed::new(Rc::clone(&push_port));

        self.handoff_connectors.push(Box::new(move |hydroflow| {
            if let Some(output_port) = push_port.take() {
                hydroflow.add_edge(output_port, input_port);
            } else {
                panic!("TCP stream output was never connected!!"); // TODO(mingwei): more informative error messages.
            }
        }));

        push
    }

    pub fn add_read_tcp_stream(&mut self, stream: tokio::net::TcpStream) -> HandoffPullSurface<VecHandoff<Message>> {
        let output_port = self.hydroflow.add_read_tcp_stream(stream);

        let pull_port = Default::default();
        let pull = HandoffPullSurface::new(Rc::clone(&pull_port));

        self.handoff_connectors.push(Box::new(move |hydroflow| {
            if let Some(input_port) = pull_port.take() {
                hydroflow.add_edge(output_port, input_port);
            } else {
                panic!("TCP stream input was never connected!!"); // TODO(mingwei): more informative error messages.
            }
        }));

        pull
    }

    pub fn add_tcp_stream(
        &mut self,
        stream: tokio::net::TcpStream,
    ) -> (HandoffPushSurfaceReversed<VecHandoff<Message>, Option<Message>>, HandoffPullSurface<VecHandoff<Message>>) {
        let (input_port, output_port) = self.hydroflow.add_tcp_stream(stream);
        
        let pull_port = Default::default();
        let pull = HandoffPullSurface::new(Rc::clone(&pull_port));

        self.handoff_connectors.push(Box::new(move |hydroflow| {
            if let Some(input_port) = pull_port.take() {
                hydroflow.add_edge(output_port, input_port);
            } else {
                panic!("TCP stream input was never connected!!"); // TODO(mingwei): more informative error messages.
            }
        }));

        let push_port = Default::default();
        let push = HandoffPushSurfaceReversed::new(Rc::clone(&push_port));

        self.handoff_connectors.push(Box::new(move |hydroflow| {
            if let Some(output_port) = push_port.take() {
                hydroflow.add_edge(output_port, input_port);
            } else {
                panic!("TCP stream output was never connected!!"); // TODO(mingwei): more informative error messages.
            }
        }));

        (push, pull)
    }

    pub fn build(mut self) -> Hydroflow {
        for handoff_connector in self.handoff_connectors {
            // TODO(mingwei): be more principled with this.
            (handoff_connector)(&mut self.hydroflow);
        }
        self.hydroflow
    }

    /// Start a new branch for teeing.
    ///
    /// For example:
    /// ```ignore
    /// my_ints
    ///     .tee(
    ///         builder
    ///             .start_tee()
    ///             .filter(|&x| 0 == x % 2)
    ///             .for_each(|x| println!("Even: {}", x)),
    ///         builder
    ///             .start_tee()
    ///             .filter(|&x| 1 == x % 2)
    ///             .for_each(|x| println!("Odd: {}", x)));
    /// ```
    pub fn start_tee<T>(&self) -> StartPushSurface<T> {
        StartPushSurface::new()
    }
}
