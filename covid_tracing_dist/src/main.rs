use clap::{ArgEnum, Parser};
use database::run_database;
use futures::{SinkExt, StreamExt};
use hydroflow::scheduled::{
    ctx::{InputPort, OutputPort, RecvCtx},
    handoff::VecHandoff,
    Hydroflow,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::TcpStream, runtime::Runtime};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracker::run_tracker;

mod database;
mod people;
mod tracker;

const MESSAGE_DATA: u8 = 0;

const TYPE_LEN: usize = 1;
const ADDRESS_LEN: usize = 4;

// This is a distributed version of the covid tracing app. It somewhat
// arbitrarily splits apart two "responsibilities" of the app to exercise
// network communication.

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    // The Database role is responsible for all the actual interfacing with the
    // external world: it knows about all the people, diagnoses, and contacts,
    // and ships off the diagnoses and contacts to the Tracker. It then receives
    // back from the tracker the IDs of people who should be notified, which are
    // then joined to look up the name and phone number of the individual.
    Database,
    // The tracker takes information from the Database and runs a reachability
    // computation on it, and sends back the IDs of people who should be
    // notified.
    Tracker,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    name: String,
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    port: usize,
    #[clap(long)]
    addr: String,
}

// TODO(justin): this trait kind of sucks but it's a placeholder.
trait Encode {
    fn encode(&self, v: &mut Vec<u8>);
}

impl<T> Encode for T
where
    T: Serialize,
{
    fn encode(&self, v: &mut Vec<u8>) {
        v.extend(serde_json::to_vec(self).unwrap());
    }
}

trait Decode {
    fn decode(v: bytes::Bytes) -> Self;
}

// TODO(justin): figure out how to do this without DeserializeOwned
impl<T> Decode for T
where
    T: 'static + DeserializeOwned,
{
    fn decode(v: bytes::Bytes) -> Self {
        let st = std::str::from_utf8(&v).unwrap().to_owned();
        serde_json::from_str(&st).unwrap()
    }
}

const CONTACTS_ADDR: u32 = 0;
const DIAGNOSES_ADDR: u32 = 1;

#[derive(Clone, Debug)]
enum Message {
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

fn add_tcp_stream(
    df: &mut Hydroflow,
    rt: Runtime,
    stream: TcpStream,
) -> (
    InputPort<VecHandoff<Message>>,
    OutputPort<VecHandoff<Message>>,
) {
    let (reader, writer) = stream.into_split();
    let reader = FramedRead::new(reader, LengthDelimitedCodec::new());
    let reader_port = df.add_input_from_stream::<_, VecHandoff<_>, _>(
        reader.map(|buf| Some(<Message>::decode(&buf.unwrap().into()))),
    );
    let mut writer = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let writer_port: InputPort<VecHandoff<Message>> =
        df.add_sink(move |recv: &RecvCtx<VecHandoff<Message>>| {
            // TODO(justin): figure out a way to eliminate this extra copy/reuse the buffer here.
            for v in recv.take_inner() {
                let mut buf = Vec::new();
                v.encode(&mut buf);
                rt.block_on(writer.send(buf.into())).unwrap();
            }
        });
    (writer_port, reader_port)
}

fn main() {
    let opts = Opts::parse();

    match opts.role {
        Role::Database => {
            run_database(opts);
        }
        Role::Tracker => {
            run_tracker(opts);
        }
    }
}
