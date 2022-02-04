use core::task;
use std::sync::mpsc::SyncSender;
use std::{pin::Pin, task::Poll};

use futures::Stream;

use super::context::Context;
use super::graph::Hydroflow;
use super::handoff::{CanReceive, Handoff};
use super::input::Input;
use super::port::{InputPort, OutputPort, RecvCtx, SendCtx};
use super::SubgraphId;

macro_rules! subgraph_ext {
    (
        $fn_name:ident,
        ( $($recv_param:ident : $recv_generic:ident),* ),
        ( $($send_param:ident : $send_generic:ident),* )
    ) => {
        fn $fn_name <F, $($recv_generic,)* $($send_generic),*>
            (&mut self, $($recv_param : OutputPort< $recv_generic >,)* $($send_param : InputPort< $send_generic >,)* subgraph: F)
            -> SubgraphId
        where
            F: 'static + FnMut(&Context<'_>, $(&RecvCtx< $recv_generic >,)* $(&SendCtx< $send_generic >),*),
            $($recv_generic : 'static + Handoff,)*
            $($send_generic : 'static + Handoff,)*;
    };
    (
        impl
        $fn_name:ident,
        ( $($recv_param:ident : $recv_generic:ident),* ),
        ( $($send_param:ident : $send_generic:ident),* )
    ) => {
        fn $fn_name <F, $($recv_generic,)* $($send_generic),*>
            (&mut self, $($recv_param : OutputPort< $recv_generic >,)* $($send_param : InputPort< $send_generic >,)* subgraph: F)
            -> SubgraphId
        where
            F: 'static + FnMut(&Context<'_>, $(&RecvCtx< $recv_generic >,)* $(&SendCtx< $send_generic >),*),
            $($recv_generic : 'static + Handoff,)*
            $($send_generic : 'static + Handoff,)*
        {
            let mut subgraph = subgraph;
            self.add_subgraph(
                crate::tl!($($recv_param),*),
                crate::tl!($($send_param),*),
                move |ctx, crate::tl!($($recv_param),*), crate::tl!($($send_param),*)|
                    (subgraph)(ctx, $($recv_param,)* $($send_param),*),
            )
        }
    };
}

/// Convenience extension methods for the Hydroflow struct.
pub trait GraphExt {
    subgraph_ext!(add_subgraph_sink, (recv_port: R), ());
    subgraph_ext!(add_subgraph_2sink, (recv_port_1: R1, recv_port_2: R2), ());

    subgraph_ext!(add_subgraph_source, (), (send_port: W));

    subgraph_ext!(add_subgraph_in_out, (recv_port: R), (send_port: W));
    subgraph_ext!(
        add_subgraph_in_2out,
        (recv_port: R),
        (send_port_1: W1, send_port_2: W2)
    );

    subgraph_ext!(
        add_subgraph_2in_out,
        (recv_port_1: R1, recv_port_2: R2),
        (send_port: W)
    );
    subgraph_ext!(
        add_subgraph_2in_2out,
        (recv_port_1: R1, recv_port_2: R2),
        (send_port_1: W1, send_port_2: W2)
    );

    fn add_channel_input<T, W>(&mut self) -> (Input<T, SyncSender<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>;

    /// Adds an "input" operator, returning a handle to insert data into it.
    /// TODO(justin): make this thing work better
    fn add_input<T, W>(&mut self) -> (Input<T, super::input::Buffer<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>;

    fn add_input_from_stream<T, W, S>(&mut self, send_port: InputPort<W>, stream: S)
    where
        S: 'static + Stream<Item = T> + Unpin,
        W: 'static + Handoff + CanReceive<T>;
}

impl GraphExt for Hydroflow {
    subgraph_ext!(impl add_subgraph_sink, (recv_port: R), ());
    subgraph_ext!(
        impl add_subgraph_2sink,
        (recv_port_1: R1, recv_port_2: R2),
        ()
    );

    subgraph_ext!(impl add_subgraph_source, (), (send_port: W));

    subgraph_ext!(impl add_subgraph_in_out, (recv_port: R), (send_port: W));
    subgraph_ext!(
        impl add_subgraph_in_2out,
        (recv_port: R),
        (send_port_1: W1, send_port_2: W2)
    );

    subgraph_ext!(
        impl add_subgraph_2in_out,
        (recv_port_1: R1, recv_port_2: R2),
        (send_port: W)
    );
    subgraph_ext!(
        impl add_subgraph_2in_2out,
        (recv_port_1: R1, recv_port_2: R2),
        (send_port_1: W1, send_port_2: W2)
    );

    fn add_channel_input<T, W>(&mut self) -> (Input<T, SyncSender<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>,
    {
        use std::sync::mpsc;

        let (sender, receiver) = mpsc::sync_channel(8000);
        let (send_port, recv_port) = self.make_edge();
        let sg_id = self.add_subgraph_source::<_, W>(send_port, move |_ctx, send| {
            for x in receiver.try_iter() {
                send.give(x);
            }
        });
        (Input::new(self.reactor(), sg_id, sender), recv_port)
    }

    fn add_input<T, W>(&mut self) -> (Input<T, super::input::Buffer<T>>, OutputPort<W>)
    where
        T: 'static,
        W: 'static + Handoff + CanReceive<T>,
    {
        let input = super::input::Buffer::default();
        let inner_input = input.clone();
        let (send_port, recv_port) = self.make_edge::<W>();
        let sg_id = self.add_subgraph_source::<_, W>(send_port, move |_ctx, send| {
            for x in (*inner_input.0).borrow_mut().drain(..) {
                send.give(x);
            }
        });
        (Input::new(self.reactor(), sg_id, input), recv_port)
    }

    fn add_input_from_stream<T, W, S>(&mut self, send_port: InputPort<W>, stream: S)
    where
        S: 'static + Stream<Item = T> + Unpin,
        W: 'static + Handoff + CanReceive<T>,
    {
        let mut stream = stream;
        self.add_subgraph_source::<_, W>(send_port, move |ctx, send| {
            let waker = ctx.waker();
            let mut cx = task::Context::from_waker(&waker);
            let mut r = Pin::new(&mut stream);
            while let Poll::Ready(Some(v)) = r.poll_next(&mut cx) {
                send.give(v);
                r = Pin::new(&mut stream);
            }
        });
    }
}
