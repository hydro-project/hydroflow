use crate::{Encodable, Message, Opts, CONTACTS_ADDR, DIAGNOSES_ADDR};

use futures::{SinkExt, StreamExt};
use hydroflow::{
    compiled::{pull::SymmetricHashJoin, ForEach, Pivot, Tee},
    scheduled::{handoff::VecHandoff, Hydroflow},
    tl, tlt,
};
use tokio::{net::TcpStream, runtime::Runtime};

use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub(crate) fn run_tracker(opts: Opts) {
    let rt = Runtime::new().unwrap();

    let stream = rt.block_on(TcpStream::connect(opts.addr)).unwrap();

    let (reader, writer) = stream.into_split();
    let reader = FramedRead::new(reader, LengthDelimitedCodec::new());

    let mut df = Hydroflow::new();

    let inputter = df.add_input_from_stream::<_, VecHandoff<_>, _>(
        reader.map(|buf| Some(<Message>::decode(&buf.unwrap().to_vec()))),
    );

    type MultiplexIn = tlt!(VecHandoff::<Message>);
    type MultiplexOut = tlt!(
        VecHandoff::<(String, String, usize)>,
        VecHandoff::<(String, (usize, usize))>,
    );

    let (tl!(demux_in), tl!(contacts, diagnoses)) = df
        .add_subgraph::<_, MultiplexIn, MultiplexOut>(move |tl!(recv), tl!(send1, send2)| {
            for message in recv.take_inner() {
                match message {
                    Message::Data { address, data } => match address {
                        CONTACTS_ADDR => {
                            send1.give(Some(<(String, String, usize)>::decode(&data)));
                        }
                        DIAGNOSES_ADDR => {
                            send2.give(Some(<(String, (usize, usize))>::decode(&data)));
                        }
                        _ => panic!("invalid port"),
                    },
                }
            }
        });

    type Pid = String;
    type DateTime = usize;

    let mut exposed_contacts = Default::default();
    type MainIn = tlt!(
        VecHandoff::<(Pid, Pid, DateTime)>,
        VecHandoff::<(Pid, (DateTime, DateTime))>,
        VecHandoff::<(Pid, DateTime)>
    );
    type MainOut = tlt!(VecHandoff::<(Pid, DateTime)>, VecHandoff::<(Pid, DateTime)>);
    let (tl!(contacts_in, diagnosed_in, loop_in), tl!(notifs_out, loop_out)) = df
        .add_subgraph::<_, MainIn, MainOut>(
            move |tl!(contacts_recv, diagnosed_recv, loop_recv), tl!(notifs_send, loop_send)| {
                let looped = loop_recv
                    .take_inner()
                    .into_iter()
                    .map(|(pid, t)| (pid, (t, t + 14)));

                let exposed = diagnosed_recv.take_inner().into_iter().chain(looped);

                let contacts =
                    contacts_recv
                        .take_inner()
                        .into_iter()
                        .flat_map(|(pid_a, pid_b, t)| {
                            vec![(pid_a.clone(), (pid_b.clone(), t)), (pid_b, (pid_a, t))]
                        });

                let join_exposed_contacts =
                    SymmetricHashJoin::new(exposed, contacts, &mut exposed_contacts);
                let new_exposed = join_exposed_contacts.filter_map(
                    |(_pid_a, (t_from, t_to), (pid_b, t_contact))| {
                        if t_from < t_contact && t_contact <= t_to {
                            Some((pid_b, t_contact))
                        } else {
                            None
                        }
                    },
                );

                let notif_push = ForEach::new(|exposed_person: (Pid, DateTime)| {
                    notifs_send.give(Some(exposed_person));
                });
                let loop_push = ForEach::new(|exposed_person: (Pid, DateTime)| {
                    loop_send.give(Some(exposed_person));
                });
                let push_exposed = Tee::new(notif_push, loop_push);

                let pivot = Pivot::new(new_exposed, push_exposed);
                pivot.run();
            },
        );

    df.add_edge(contacts, contacts_in);
    df.add_edge(diagnoses, diagnosed_in);
    df.add_edge(loop_out, loop_in);

    let mut writer = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let send_back = df.add_sink(move |recv| {
        for v in recv.take_inner() {
            let mut buf = Vec::new();
            Encodable::encode(&v, &mut buf);
            rt.block_on(writer.send(buf.into())).unwrap();
        }
    });

    df.add_edge(notifs_out, send_back);
    df.add_edge(inputter, demux_in);

    df.run().unwrap();
}
