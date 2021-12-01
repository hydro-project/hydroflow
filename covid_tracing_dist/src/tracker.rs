use crate::{Decode, Encode, Opts, CONTACTS_ADDR, DIAGNOSES_ADDR};

use hydroflow::{
    compiled::{pull::SymmetricHashJoin, ForEach, Pivot, Tee},
    scheduled::{
        collections::Iter,
        ctx::RecvCtx,
        handoff::VecHandoff,
        net::{Message, Net},
        Hydroflow,
    },
    tl, tlt,
};

pub(crate) fn run_tracker(opts: Opts) {
    let mut df = Hydroflow::new();

    type MultiplexIn = tlt!(VecHandoff::<Message>);
    type MultiplexOut = tlt!(
        VecHandoff::<(String, String, usize)>,
        VecHandoff::<(String, (usize, usize))>,
    );

    let (tl!(demux_in), tl!(contacts, diagnoses)) = df
        .add_subgraph::<_, MultiplexIn, MultiplexOut>(move |_ctx, tl!(recv), tl!(send1, send2)| {
            for message in recv.take_inner() {
                match message {
                    Message::Data { address, batch } => match address {
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
            move |_ctx,
                  tl!(contacts_recv, diagnosed_recv, loop_recv),
                  tl!(notifs_send, loop_send)| {
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

    let (encoder_in, encoder_out) =
        df.add_inout(|_ctx, recv: &RecvCtx<VecHandoff<(String, usize)>>, send| {
            let mut buf = Vec::new();
            recv.take_inner().encode(&mut buf);
            send.give(Some(Message::Data {
                address: 0,
                batch: buf.into(),
            }));
        });

    df.add_edge(contacts, contacts_in);
    df.add_edge(diagnoses, diagnosed_in);
    df.add_edge(loop_out, loop_in);

    let (network_out, network_in) = df.connect(opts.addr.as_str());

    df.add_edge(notifs_out, encoder_in);
    df.add_edge(encoder_out, network_out);
    df.add_edge(network_in, demux_in);

    df.run().unwrap();
}
