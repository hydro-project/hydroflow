use crate::{Decode, Encode, Opts, CONTACTS_ADDR, DIAGNOSES_ADDR};

use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::{graph::Hydroflow, handoff::VecHandoff, net::Message};
use hydroflow::tokio::net::TcpStream;
use hydroflow::{
    compiled::{pull::SymmetricHashJoin, InputBuild, IteratorToPusherator, PusheratorBuild},
    scheduled::graph_ext::GraphExt,
    tl,
};

pub(crate) async fn run_tracker(opts: Opts) {
    let mut df = Hydroflow::new();

    let stream = TcpStream::connect(opts.addr).await.unwrap();
    let (network_out, network_in) = df.add_tcp_stream(stream);

    let (contacts, contacts_in) = df.make_edge::<VecHandoff<(String, String, usize)>>();
    let (diagnoses, diagnosed_in) = df.make_edge::<VecHandoff<(String, (usize, usize))>>();
    let (loop_out, loop_in) = df.make_edge::<VecHandoff<(Pid, DateTime)>>();
    let (notifs_out, encoder_in) = df.make_edge::<VecHandoff<(Pid, DateTime)>>();

    df.add_subgraph(
        tl!(network_in),
        tl!(contacts, diagnoses),
        move |_ctx, tl!(recv), tl!(send1, send2)| {
            for message in recv.take_inner() {
                let Message { address, batch } = message;
                match address {
                    CONTACTS_ADDR => {
                        send1.give(Iter(
                            <Vec<(String, String, usize)>>::decode(batch).into_iter(),
                        ));
                    }
                    DIAGNOSES_ADDR => {
                        send2.give(Iter(
                            <Vec<(String, (usize, usize))>>::decode(batch).into_iter(),
                        ));
                    }
                    _ => panic!("invalid port"),
                }
            }
        },
    );

    type Pid = String;
    type DateTime = usize;

    let mut exposed_contacts = Default::default();
    df.add_subgraph(
        tl!(contacts_in, diagnosed_in, loop_in),
        tl!(notifs_out, loop_out),
        move |_ctx, tl!(contacts_recv, diagnosed_recv, loop_recv), tl!(notifs_send, loop_send)| {
            let looped = loop_recv
                .take_inner()
                .into_iter()
                .map(|(pid, t)| (pid, (t, t + 14)));

            let exposed = diagnosed_recv.take_inner().into_iter().chain(looped);

            let contacts = contacts_recv
                .take_inner()
                .into_iter()
                .flat_map(|(pid_a, pid_b, t)| {
                    vec![(pid_a.clone(), (pid_b.clone(), t)), (pid_b, (pid_a, t))]
                });

            let join_exposed_contacts =
                SymmetricHashJoin::new(exposed, contacts, &mut exposed_contacts);
            let new_exposed =
                join_exposed_contacts.filter_map(|(_pid_a, (t_from, t_to), (pid_b, t_contact))| {
                    if t_from < t_contact && t_contact <= t_to {
                        Some((pid_b, t_contact))
                    } else {
                        None
                    }
                });

            let pivot = new_exposed
                .pusherator()
                .tee(
                    InputBuild::new().for_each(|exposed_person: (Pid, DateTime)| {
                        // Notif push.
                        notifs_send.give(Some(exposed_person));
                    }),
                )
                .for_each(|exposed_person: (Pid, DateTime)| {
                    // Loop push.
                    loop_send.give(Some(exposed_person));
                });

            pivot.run();
        },
    );

    df.add_subgraph_in_out(encoder_in, network_out, |_ctx, recv, send| {
        let mut buf = Vec::new();
        recv.take_inner().encode(&mut buf);
        send.give(Some(Message {
            address: 0,
            batch: buf.into(),
        }));
    });

    df.run_async().await.unwrap();
}
