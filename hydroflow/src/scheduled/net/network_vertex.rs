use std::collections::HashMap;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::scheduled::{
    ctx::{InputPort, OutputPort},
    graph::Hydroflow,
    graph_ext::GraphExt,
    handoff::VecHandoff,
};

use super::Message;

pub type Address = String;

// These methods can't be wrapped up in a trait because async methods are not
// allowed in traits (yet).

impl Hydroflow {
    // Listen on a port and send any messages received along the output edge.
    // Returns the port bound to.
    pub async fn inbound_tcp_vertex(&mut self) -> (u16, OutputPort<VecHandoff<Message>>) {
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // TODO(justin): figure out an appropriate buffer here.
        let (incoming_send, incoming_messages) = futures::channel::mpsc::channel(1024);

        // Listen to incoming connections and spawn a tokio task for each one,
        // which feeds into the channel.
        // TODO(justin): give some way to get a handle into this thing.
        tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let (reader, _) = socket.into_split();
                let mut reader = FramedRead::new(reader, LengthDelimitedCodec::new());
                let mut incoming_send = incoming_send.clone();
                tokio::spawn(async move {
                    while let Some(msg) = reader.next().await {
                        // TODO(justin): figure out error handling here.
                        let msg = msg.unwrap();
                        let msg = Message::decode(&msg.freeze());
                        incoming_send.send(msg).await.unwrap();
                    }
                    // TODO(justin): The connection is closed, so we should
                    // clean up its metadata somehow and issue a retraction for
                    // the connection.
                });
            }
        });

        let incoming_messages = self.add_input_from_stream(incoming_messages.map(Some));

        (port, incoming_messages)
    }

    // Create a TCP vertex which allows sending messages to a network address.
    pub async fn outbound_tcp_vertex(&mut self) -> InputPort<VecHandoff<(Address, Message)>> {
        let (mut connection_reqs_send, mut connection_reqs_recv) =
            futures::channel::mpsc::channel(1024);
        let (mut connections_send, mut connections_recv) = futures::channel::mpsc::channel(1024);

        // TODO(justin): handle errors here.
        // Spawn an actor which establishes connections.
        tokio::spawn(async move {
            while let Some(addr) = connection_reqs_recv.next().await {
                let addr: Address = addr;
                let stream = TcpStream::connect(addr.clone()).await.unwrap();
                connections_send.send((addr, stream)).await.unwrap();
            }
        });

        enum ConnStatus {
            Pending(Vec<Message>),
            Connected(FramedWrite<TcpStream, LengthDelimitedCodec>),
        }

        let (mut outbound_messages_send, mut outbound_messages_recv) =
            futures::channel::mpsc::channel(1024);
        tokio::spawn(async move {
            // TODO(justin): this cache should be global to the entire Hydroflow
            // instance so we can reuse connections from inbound connections.
            let mut connections = HashMap::<Address, ConnStatus>::new();

            loop {
                tokio::select! {
                    Some((addr, msg)) = outbound_messages_recv.next() => {
                        let addr: Address = addr;
                        let msg: Message = msg;
                        match connections.get_mut(&addr) {
                            None => {
                                // We have not seen this address before, open a
                                // connection to it and buffer the message to be
                                // sent once it's open.

                                // TODO(justin): what do we do if the buffer is full here?
                                connection_reqs_send.try_send(addr.clone()).unwrap();
                                connections.insert(addr, ConnStatus::Pending(vec![msg]));
                            }
                            Some(ConnStatus::Pending(msgs)) => {
                                // We have seen this address before but we're
                                // still trying to connect to it, so buffer this
                                // message so that when we _do_ connect we will
                                // send it.
                                msgs.push(msg);
                            }
                            Some(ConnStatus::Connected(conn)) => {
                                // TODO(justin): move the actual sending here
                                // into a different task so we don't have to
                                // wait for the send.
                                let mut data = Vec::new();
                                msg.encode(&mut data);
                                conn.send(data.into()).await.unwrap();
                            }
                        }
                    },

                    Some((addr, conn)) = connections_recv.next() => {
                        match connections.get_mut(&addr) {
                            Some(ConnStatus::Pending(msgs)) => {
                                let mut conn = FramedWrite::new(conn, LengthDelimitedCodec::new());
                                for msg in msgs.drain(..) {
                                    // TODO(justin): move the actual sending here
                                    // into a different task so we don't have to
                                    // wait for the send.
                                    let mut data = Vec::new();
                                    msg.encode(&mut data);
                                    conn.send(data.into()).await.unwrap();
                                }
                                connections.insert(addr, ConnStatus::Connected(conn));
                            }
                            None => {
                                // This means nobody ever requested this
                                // connection, so we shouldn't have initiated it
                                // in the first place.
                                unreachable!()
                            }
                            Some(ConnStatus::Connected(_tcp)) => {
                                // This means we were already connected, so we
                                // shouldn't have connected again. If the
                                // connection cache becomes shared this could
                                // become reachable.
                                unreachable!()
                            }
                        }
                    },
                    else => break,
                }
            }
        });

        let mut buffered_messages = Vec::new();
        let mut next_messages = Vec::new();
        let input_port = self.add_sink(move |_ctx, recv| {
            buffered_messages.extend(recv.take_inner());
            for msg in buffered_messages.drain(..) {
                if let Err(e) = outbound_messages_send.try_send(msg) {
                    // If we weren't able to send a message (say, because the
                    // buffer is full), we get handed it back in the error. If
                    // this happens we hang onto the message to try sending it
                    // again next time.
                    next_messages.push(e.into_inner());
                }
            }

            // NB. we don't need to flush the channel here due to the use of
            // `try_send`.  It's guaranteed that there was space for the
            // messages and that they were sent.

            // TODO(justin): we do need to make sure we get rescheduled if
            // next_messages is empty here.

            std::mem::swap(&mut buffered_messages, &mut next_messages);
        });
        input_port
    }
}
