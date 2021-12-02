use std::collections::VecDeque;
use std::pin::Pin;

use futures::{Sink, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::{
    ctx::{InputPort, OutputPort, RecvCtx},
    handoff::VecHandoff,
    Hydroflow,
};

const MESSAGE_DATA: u8 = 0;

const TYPE_LEN: usize = 1;
const ADDRESS_LEN: usize = 4;

#[derive(Clone, Debug)]
pub enum Message {
    Data { address: u32, batch: bytes::Bytes },
}

impl Message {
    fn encode(&self, v: &mut Vec<u8>) {
        match self {
            Message::Data { address, batch } => {
                v.push(MESSAGE_DATA);
                v.extend((*address as u32).to_ne_bytes());
                v.extend(batch);
            }
        }
    }

    fn decode(v: &bytes::Bytes) -> Self {
        match v[0] {
            MESSAGE_DATA => {
                let address =
                    u32::from_ne_bytes(v[TYPE_LEN..(TYPE_LEN + ADDRESS_LEN)].try_into().unwrap());
                let batch = v.slice((TYPE_LEN + ADDRESS_LEN)..);
                Message::Data { address, batch }
            }
            _ => panic!("unhandled"),
        }
    }
}

impl Hydroflow {
    fn add_tcp_stream(
        &mut self,
        stream: TcpStream,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    ) {
        let (reader, writer) = stream.into_split();
        let reader = FramedRead::new(reader, LengthDelimitedCodec::new());
        let reader_port = self.add_input_from_stream::<_, VecHandoff<_>, _>(
            reader.map(|buf| Some(<Message>::decode(&buf.unwrap().into()))),
        );
        let mut writer = FramedWrite::new(writer, LengthDelimitedCodec::new());

        let mut message_queue = VecDeque::new();

        let writer_port: InputPort<VecHandoff<Message>> =
            self.add_sink(move |ctx, recv: &RecvCtx<VecHandoff<Message>>| {
                let waker = ctx.waker();
                let mut cx = std::task::Context::from_waker(&waker);

                // TODO(mingwei): queue may grow unbounded? Subtle rate matching concern.
                // TODO(mingwei): put into state system.
                message_queue.extend(recv.take_inner().into_iter());
                while !message_queue.is_empty() {
                    if let std::task::Poll::Ready(Ok(())) =
                        Pin::new(&mut writer).poll_ready(&mut cx)
                    {
                        let v = message_queue.pop_front().unwrap();
                        let mut buf = Vec::new();
                        v.encode(&mut buf);

                        Pin::new(&mut writer).start_send(buf.into()).unwrap();
                    }
                }
                let _ = Pin::new(&mut writer).poll_flush(&mut cx);
            });
        (writer_port, reader_port)
    }

    // Connects to the specified address, returning an input and output port
    // allowing communication on it.
    pub async fn connect(
        &mut self,
        addr: &str,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    ) {
        let stream = TcpStream::connect(addr).await.unwrap();
        self.add_tcp_stream(stream)
    }

    // Waits for a single connection on the specified unix port, returning an input
    // and output port allowing communication on it.
    pub async fn bind_one(
        &mut self,
        port: usize,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    ) {
        let stream = TcpListener::bind(format!("localhost:{}", port))
            .await
            .unwrap();
        let (stream, _) = stream.accept().await.unwrap();
        self.add_tcp_stream(stream)
    }
}
