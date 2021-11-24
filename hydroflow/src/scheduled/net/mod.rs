use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::{
    ctx::{InputPort, OutputPort, RecvCtx},
    handoff::VecHandoff,
    Hydroflow,
};

// Networking extensions for the Hydroflow struct.
pub trait Net {
    fn connect(
        &mut self,
        addr: &str,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    );

    fn bind_one(
        &mut self,
        port: usize,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    );
}

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
        let rt = self.rt.clone();
        let writer_port: InputPort<VecHandoff<Message>> =
            self.add_sink(move |recv: &RecvCtx<VecHandoff<Message>>| {
                // TODO(justin): figure out a way to eliminate this extra copy/reuse the buffer here.
                for v in recv.take_inner() {
                    let mut buf = Vec::new();
                    v.encode(&mut buf);
                    (*rt).block_on(writer.send(buf.into())).unwrap();
                }
            });
        (writer_port, reader_port)
    }
}

impl Net for Hydroflow {
    // Connects to the specified address, returning an input and output port
    // allowing communication on it.
    fn connect(
        &mut self,
        addr: &str,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    ) {
        let stream = (*self.rt).block_on(TcpStream::connect(addr)).unwrap();
        self.add_tcp_stream(stream)
    }

    // Waits for a single connection on the specified unix port, returning an input
    // and output port allowing communication on it.
    fn bind_one(
        &mut self,
        port: usize,
    ) -> (
        InputPort<VecHandoff<Message>>,
        OutputPort<VecHandoff<Message>>,
    ) {
        let stream = (*self.rt)
            .block_on(TcpListener::bind(format!("localhost:{}", port)))
            .unwrap();
        let (stream, _) = (*self.rt).block_on(stream.accept()).unwrap();

        self.add_tcp_stream(stream)
    }
}
