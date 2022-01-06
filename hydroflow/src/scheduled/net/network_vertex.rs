use std::{collections::HashMap, pin::Pin};

use futures::{Sink, StreamExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use crate::scheduled::{
    ctx::{InputPort, RecvCtx},
    graph::Hydroflow,
    graph_ext::GraphExt,
    handoff::VecHandoff,
};

use super::Message;

pub type ReceiverId = u32;

pub type OutboundMessage = (ReceiverId, Message);

impl Hydroflow {
    // TODO(justin): Add docs here once the semantics are a bit more nailed down.
    pub fn add_egress_vertex(
        &mut self,
    ) -> (
        futures::channel::mpsc::Sender<(ReceiverId, OwnedWriteHalf)>,
        InputPort<VecHandoff<OutboundMessage>>,
    ) {
        let (send_conns, recv_conns) = futures::channel::mpsc::channel(1024);
        // TODO(justin):  this should perhaps be some kind of cache? Or a
        // slot_map that allows us to remove connections.
        let mut conns: HashMap<ReceiverId, FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>> =
            HashMap::new();

        let conns_out = self.add_input_from_stream(recv_conns.map(Some));

        // Work on the assumption that most of the time, if we sent a message to
        // an address we don't know about, we'll learn about it soon.
        let mut buffered_messages = Vec::new();
        let mut swap_buffer = Vec::new();

        let (conns_in, sends_in) = self.add_binary_sink(
            move |ctx, recv_conn: &RecvCtx<VecHandoff<_>>, recv_msg: &RecvCtx<VecHandoff<_>>| {
                for (id, ch) in recv_conn.take_inner() {
                    // TODO(justin): complain if there's a duplicate id?
                    conns.insert(id, FramedWrite::new(ch, LengthDelimitedCodec::new()));
                }

                let waker = ctx.waker();
                let mut cx = std::task::Context::from_waker(&waker);

                buffered_messages.extend(recv_msg.take_inner().drain(..));

                for (id, msg) in buffered_messages.drain(..) {
                    let msg: Message = msg;
                    if let Some(mut writer) = conns.get_mut(&id) {
                        // TODO(mingwei): queue may grow unbounded? Subtle rate matching concern.
                        // TODO(mingwei): put into state system.
                        if let std::task::Poll::Ready(Ok(())) =
                            Pin::new(&mut writer).poll_ready(&mut cx)
                        {
                            let mut data = Vec::new();
                            msg.encode(&mut data);
                            Pin::new(&mut writer).start_send(data.into()).unwrap();
                        }
                        let _ = Pin::new(&mut writer).poll_flush(&mut cx);
                    } else {
                        // If we didn't have somewhere to send this message,
                        // queue it up to try again next time we are scheduled
                        // (which will happen if we learn about a new address).
                        swap_buffer.push((id, msg));
                    }
                }

                std::mem::swap(&mut buffered_messages, &mut swap_buffer);
            },
        );

        self.add_edge(conns_out, conns_in);

        (send_conns, sends_in)
    }
}
