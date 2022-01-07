use std::{collections::HashMap, pin::Pin};

use futures::{Sink, SinkExt, StreamExt};
use tokio::net::{tcp::OwnedWriteHalf, TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{
    lang::collections::Iter,
    scheduled::{
        ctx::{InputPort, OutputPort, RecvCtx},
        graph::Hydroflow,
        graph_ext::GraphExt,
        handoff::VecHandoff,
    },
};

use super::Message;

pub type ReceiverId = u32;

pub type OutboundMessage = (ReceiverId, Message);

impl Hydroflow {
    // TODO(justin): this needs to return a result/get rid of all the unwraps, I
    // guess we need a HydroflowError?
    pub async fn listen_tcp(
        &mut self,
    ) -> (
        u16,
        OutputPort<VecHandoff<(ReceiverId, Message)>>,
        OutputPort<VecHandoff<ReceiverId>>,
        InputPort<VecHandoff<(u32, Message)>>,
    ) {
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (mut conns, egress_edge) = self.add_egress_vertex();
        // TODO(justin): figure out an appropriate buffer here.
        let (incoming_send, incoming_messages) = futures::channel::mpsc::channel(1024);
        let (mut incoming_conns, conns_recv) = futures::channel::mpsc::channel(1024);

        // Listen to incoming connections and spawn a tokio task for each one,
        // which feeds into the channel.
        // TODO(justin): give some way to get a handle into this thing.
        tokio::spawn(async move {
            let mut conn_id = 0;
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let (reader, writer) = socket.into_split();
                conns.feed((conn_id, writer)).await.unwrap();
                conns.flush().await.unwrap();
                let mut reader = FramedRead::new(reader, LengthDelimitedCodec::new());
                let mut incoming_send = incoming_send.clone();
                let id = conn_id;
                incoming_conns.feed(conn_id).await.unwrap();
                incoming_conns.flush().await.unwrap();
                tokio::spawn(async move {
                    while let Some(msg) = reader.next().await {
                        // TODO(justin): figure out error handling here.
                        let msg = msg.unwrap();
                        let msg = Message::decode(&msg.freeze());
                        incoming_send.feed((id, msg)).await.unwrap();
                    }
                    // TODO(justin): The connection is closed, so we should
                    // clean up its metadata somehow and issue a retraction for
                    // the connection.
                });
                conn_id += 1;
            }
        });

        let incoming_messages = self.add_input_from_stream(incoming_messages.map(Some));
        let conns_recv = self.add_input_from_stream(conns_recv.map(Some));

        (port, incoming_messages, conns_recv, egress_edge)
    }

    // TODO(justin): we should have a vertex that can dynamically connect to
    // stuff based on data instead of this.
    pub async fn connect_tcp<A>(
        &mut self,
        addr: A,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    )
    where
        A: ToSocketAddrs,
    {
        let stream = TcpStream::connect(addr).await.unwrap();

        let (reader, writer) = stream.into_split();

        let (mut conns, egress_edge) = self.add_egress_vertex();
        conns.feed((0, writer)).await.unwrap();
        conns.flush().await.unwrap();

        let (result_in, send) = self.add_inout(|_ctx, recv: &RecvCtx<VecHandoff<_>>, send| {
            send.give(Iter(recv.take_inner().into_iter().map(|msg| (0, msg))));
        });

        self.add_edge(send, egress_edge);

        let incoming_messages = self.add_input_from_stream(
            FramedRead::new(reader, LengthDelimitedCodec::new())
                .map(|msg| Message::decode(&msg.unwrap().freeze()))
                .map(Some),
        );
        (result_in, incoming_messages)
    }

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
                            // TODO(justin): need to handle if the connection has closed.
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
