use crate::{people, Decode, Encode, Opts, CONTACTS_ADDR, DIAGNOSES_ADDR};

use std::time::Duration;

use hydroflow::compiled::{pull::SymmetricHashJoin, IteratorToPusherator, PusheratorBuild};
use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::{handoff::VecHandoff, net::Message};
use hydroflow::tokio::net::TcpListener;
use hydroflow::{
    scheduled::{graph::Hydroflow, graph_ext::GraphExt},
    tl,
};
use rand::Rng;

pub(crate) async fn run_database(opts: Opts) {
    let all_people = people::get_people();

    let mut df = Hydroflow::new();

    let (notifs, notif_sink) = df.make_edge::<VecHandoff<(String, usize)>>("notifs".into());
    let (encode_contacts_out, contacts_merge) =
        df.make_edge::<VecHandoff<Message>>("encoded contacts".into());
    let (encode_diagnoses_out, diagnoses_merge) =
        df.make_edge::<VecHandoff<Message>>("encoded diagnoses".into());

    let (contacts_send, contacts_recv) =
        df.make_edge::<VecHandoff<(&'static str, &'static str, usize)>>("contacts".into());
    let contacts_send = df.add_channel_input("contacts input".into(), contacts_send);
    let (diagnosed_send, diagnosed_recv) =
        df.make_edge::<VecHandoff<(&'static str, (usize, usize))>>("diagnosed".into());
    let diagnosed_send = df.add_channel_input("diagnosed input".into(), diagnosed_send);
    let (people_send, people_recv) =
        df.make_edge::<VecHandoff<(String, (String, String))>>("people".into());
    let people_send = df.add_channel_input("people input".into(), people_send);

    let stream = TcpListener::bind(format!("localhost:{}", opts.port))
        .await
        .unwrap();

    let (stream, _) = stream.accept().await.unwrap();
    let (network_in, network_out) = df.add_tcp_stream(stream);

    df.add_subgraph_in_out(
        "decode messages".into(),
        network_out,
        notifs,
        |_ctx, recv, send| {
            for message in recv.take_inner().into_iter() {
                send.give(Iter(
                    <Vec<(String, usize)>>::decode(message.batch).into_iter(),
                ));
            }
        },
    );

    std::thread::spawn(move || {
        let mut t = 0;
        let mut rng = rand::thread_rng();
        for (id, (name, phone)) in all_people.clone() {
            people_send.give(Some((id.to_owned(), (name.to_owned(), phone.to_owned()))));
        }
        people_send.flush();
        loop {
            t += 1;
            match rng.gen_range(0..2) as usize {
                0 => {
                    // New contact.
                    if all_people.len() >= 2 {
                        let p1 = rng.gen_range(0..all_people.len());
                        let p2 = rng.gen_range(0..all_people.len());
                        if p1 != p2 {
                            contacts_send.give(Some((all_people[p1].0, all_people[p2].0, t)));
                            contacts_send.flush();
                        }
                    }
                }
                1 => {
                    // Diagnosis.
                    if !all_people.is_empty() {
                        let p = rng.gen_range(0..all_people.len());
                        diagnosed_send.give(Some((all_people[p].0, (t, t + 14))));
                        diagnosed_send.flush();
                    }
                }
                _ => unreachable!(),
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    df.add_subgraph_2in_out(
        "merge contacts and diagnoses".into(),
        contacts_merge,
        diagnoses_merge,
        network_in,
        |_ctx, recv1, recv2, send| {
            send.give(Iter(
                recv1.take_inner().into_iter().chain(recv2.take_inner()),
            ));
        },
    );

    df.add_subgraph_in_out(
        "encode contacts".into(),
        contacts_recv,
        encode_contacts_out,
        |_ctx, recv, send| {
            let mut buf = Vec::new();
            recv.take_inner().encode(&mut buf);
            send.give(Some(Message {
                address: CONTACTS_ADDR,
                batch: buf.into(),
            }));
        },
    );

    df.add_subgraph_in_out(
        "encode diagnoses".into(),
        diagnosed_recv,
        encode_diagnoses_out,
        |_ctx, recv, send| {
            let mut buf = Vec::new();
            recv.take_inner().encode(&mut buf);
            send.give(Some(Message {
                address: DIAGNOSES_ADDR,
                batch: buf.into(),
            }));
        },
    );

    let mut join_state = Default::default();
    df.add_subgraph(
        "join people and notifs".into(),
        tl!(notif_sink, people_recv),
        tl!(),
        move |_ctx, tl!(notifs, people), tl!()| {
            let pivot = SymmetricHashJoin::new(
                notifs.take_inner().into_iter(),
                people.take_inner().into_iter(),
                &mut join_state,
            )
            .map(|(_id, t, (name, phone))| (name, phone, t))
            .pull_to_push()
            .for_each(|(name, phone, t)| println!("notifying {}, {}@{}", name, phone, t));

            pivot.run();
        },
    );

    df.run_async().await.unwrap();
}
