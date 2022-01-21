use core::task;
use std::{pin::Pin, sync::mpsc::SyncSender, task::Poll};

use futures::Stream;

use super::{
    context::Context,
    ctx::{InputPort, OutputPort, RecvCtx, SendCtx},
    graph::Hydroflow,
    handoff::{CanReceive, Handoff},
    input::Input,
};
use crate::tl;

// Convenience extension methods for the Hydroflow struct.

pub trait GraphExt {
    /**
     * Adds a new compiled subgraph with a single input and output, and returns the input/output handles.
     */
    fn add_inout<F, R, W>(&mut self, subgraph: F) -> (InputPort<R>, OutputPort<W>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R>, &SendCtx<W>),
        R: 'static + Handoff,
        W: 'static + Handoff;

    /**
     * Adds a new compiled subgraph with one input and two outputs, and returns the input/output handles.
     */
    fn add_binary_out<F, R, W1, W2>(
        &mut self,
        subgraph: F,
    ) -> (InputPort<R>, OutputPort<W1>, OutputPort<W2>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R>, &SendCtx<W1>, &SendCtx<W2>),
        R: 'static + Handoff,
        W1: 'static + Handoff,
        W2: 'static + Handoff;

    /**
     * Adds a new compiled subgraph with one input and two outputs, and returns the input/output handles.
     */
    fn add_binary_in_binary_out<F, R1, R2, W1, W2>(
        &mut self,
        subgraph: F,
    ) -> (InputPort<R1>, InputPort<R2>, OutputPort<W1>, OutputPort<W2>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R1>, &RecvCtx<R2>, &SendCtx<W1>, &SendCtx<W2>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
        W1: 'static + Handoff,
        W2: 'static + Handoff;

    /**
     * Adds a new compiled subraph with two inputs and a single output, and returns the input/output handles.
     */
    fn add_binary<F, R1, R2, W>(
        &mut self,
        subgraph: F,
    ) -> (InputPort<R1>, InputPort<R2>, OutputPort<W>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R1>, &RecvCtx<R2>, &SendCtx<W>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
        W: 'static + Handoff;

    /**
     * Adds a new compiled subraph with two inputs and no outputs, and returns the input handles.
     */
    fn add_binary_sink<F, R1, R2>(&mut self, subgraph: F) -> (InputPort<R1>, InputPort<R2>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R1>, &RecvCtx<R2>),
        R1: 'static + Handoff,
        R2: 'static + Handoff;

    /**
     * Adds a new compiled subgraph with one input and no outputs.
     */
    fn add_sink<F, R>(&mut self, subgraph: F) -> InputPort<R>
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R>),
        R: 'static + Handoff;

    /**
     * Adds a new compiled subgraph with one output and no inputs.
     */
    fn add_source<F, W>(&mut self, subgraph: F) -> OutputPort<W>
    where
        F: 'static + FnMut(&Context<'_>, &SendCtx<W>),
        W: 'static + Handoff;

    fn add_channel_input<T, W>(&mut self) -> (Input<T, SyncSender<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>;

    /**
     * Adds an "input" operator, returning a handle to insert data into it.
     * TODO(justin): make this thing work better
     */
    fn add_input<T, W>(&mut self) -> (Input<T, super::input::Buffer<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>;

    fn add_input_from_stream<T, W, S>(&mut self, s: S) -> OutputPort<W>
    where
        S: 'static + Stream<Item = T> + Unpin,
        W: 'static + Handoff + CanReceive<T>;
}

impl GraphExt for Hydroflow {
    #[cfg(feature = "variadic_generics")]
    fn add_inout<F, R, W>(&mut self, mut subgraph: F) -> (InputPort<R>, OutputPort<W>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R>, &SendCtx<W>),
        R: 'static + Handoff,
        W: 'static + Handoff,
    {
        let (tl!(input_port), tl!(output_port)) =
            self.add_subgraph::<_, tl!(R), tl!(W)>(move |ctx, tl!(recv), tl!(send)| {
                (subgraph)(ctx, recv, send)
            });
        (input_port, output_port)
    }

    fn add_binary_out<F, R, W1, W2>(
        &mut self,
        mut subgraph: F,
    ) -> (InputPort<R>, OutputPort<W1>, OutputPort<W2>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R>, &SendCtx<W1>, &SendCtx<W2>),
        R: 'static + Handoff,
        W1: 'static + Handoff,
        W2: 'static + Handoff,
    {
        let (tl!(input_port), tl!(output_port1, output_port2)) = self
            .add_subgraph::<_, tl!(R), tl!(W1, W2)>(move |ctx, tl!(recv), tl!(send1, send2)| {
                (subgraph)(ctx, recv, send1, send2)
            });
        (input_port, output_port1, output_port2)
    }

    fn add_binary_in_binary_out<F, R1, R2, W1, W2>(
        &mut self,
        mut subgraph: F,
    ) -> (InputPort<R1>, InputPort<R2>, OutputPort<W1>, OutputPort<W2>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R1>, &RecvCtx<R2>, &SendCtx<W1>, &SendCtx<W2>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
        W1: 'static + Handoff,
        W2: 'static + Handoff,
    {
        let (tl!(input_port1, input_port2), tl!(output_port1, output_port2)) = self
            .add_subgraph::<_, tl!(R1, R2), tl!(W1, W2)>(
                move |ctx, tl!(recv1, recv2), tl!(send1, send2)| {
                    (subgraph)(ctx, recv1, recv2, send1, send2)
                },
            );
        (input_port1, input_port2, output_port1, output_port2)
    }

    /**
     * Adds a new compiled subraph with two inputs and a single output, and returns the input/output handles.
     */
    #[cfg(feature = "variadic_generics")]
    fn add_binary<F, R1, R2, W>(
        &mut self,
        mut subgraph: F,
    ) -> (InputPort<R1>, InputPort<R2>, OutputPort<W>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R1>, &RecvCtx<R2>, &SendCtx<W>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
        W: 'static + Handoff,
    {
        let (tl!(input_port1, input_port2), tl!(output_port)) = self
            .add_subgraph::<_, tl!(R1, R2), tl!(W)>(move |ctx, tl!(recv1, recv2), tl!(send)| {
                (subgraph)(ctx, recv1, recv2, send)
            });
        (input_port1, input_port2, output_port)
    }

    #[cfg(feature = "variadic_generics")]
    fn add_binary_sink<F, R1, R2>(&mut self, mut subgraph: F) -> (InputPort<R1>, InputPort<R2>)
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R1>, &RecvCtx<R2>),
        R1: 'static + Handoff,
        R2: 'static + Handoff,
    {
        let (tl!(input_port1, input_port2), tl!()) =
            self.add_subgraph::<_, tl!(R1, R2), tl!()>(move |ctx, tl!(recv1, recv2), tl!()| {
                (subgraph)(ctx, recv1, recv2)
            });
        (input_port1, input_port2)
    }

    #[cfg(feature = "variadic_generics")]
    fn add_sink<F, R>(&mut self, mut subgraph: F) -> InputPort<R>
    where
        F: 'static + FnMut(&Context<'_>, &RecvCtx<R>),
        R: 'static + Handoff,
    {
        let (tl!(input_port), tl!()) = self
            .add_subgraph::<_, tl!(R), tl!()>(move |ctx, tl!(recv), tl!()| (subgraph)(ctx, recv));
        input_port
    }

    #[cfg(feature = "variadic_generics")]
    fn add_source<F, W>(&mut self, mut subgraph: F) -> OutputPort<W>
    where
        F: 'static + FnMut(&Context<'_>, &SendCtx<W>),
        W: 'static + Handoff,
    {
        let (tl!(), tl!(output_port)) = self
            .add_subgraph::<_, tl!(), tl!(W)>(move |ctx, tl!(), tl!(send)| (subgraph)(ctx, send));
        output_port
    }

    #[cfg(feature = "variadic_generics")]
    fn add_channel_input<T, W>(&mut self) -> (Input<T, SyncSender<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>,
    {
        use std::sync::mpsc;

        let (sender, receiver) = mpsc::sync_channel(8000);
        let output_port = self.add_source::<_, W>(move |_ctx, send| {
            for x in receiver.try_iter() {
                send.give(x);
            }
        });
        let id = output_port.sg_id;
        (Input::new(self.reactor(), id, sender), output_port)
    }

    #[cfg(feature = "variadic_generics")]
    fn add_input<T, W>(&mut self) -> (Input<T, super::input::Buffer<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>,
    {
        let input = super::input::Buffer::default();
        let inner_input = input.clone();
        let output_port = self.add_source::<_, W>(move |_ctx, send| {
            for x in (*inner_input.0).borrow_mut().drain(..) {
                send.give(x);
            }
        });
        let id = output_port.sg_id;
        (Input::new(self.reactor(), id, input), output_port)
    }

    fn add_input_from_stream<T, W, S>(&mut self, mut s: S) -> OutputPort<W>
    where
        S: 'static + Stream<Item = T> + Unpin,
        W: 'static + Handoff + CanReceive<T>,
    {
        let output_port = self.add_source::<_, W>(move |ctx, send| {
            let waker = ctx.waker();
            let mut cx = task::Context::from_waker(&waker);
            let mut r = Pin::new(&mut s);
            while let Poll::Ready(Some(v)) = r.poll_next(&mut cx) {
                send.give(v);
                r = Pin::new(&mut s);
            }
        });
        output_port
    }
}
