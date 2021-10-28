use hydroflow::compiled::ForEach;
use hydroflow::compiled::Pivot;
use hydroflow::compiled::Tee;
use std::sync::mpsc;
// use std::time::Duration;

// use time::Date;
// use time::Month;
// use time::PrimitiveDateTime;
// use time::Time;

use hydroflow::compiled::pull::SymmetricHashJoin;
use hydroflow::scheduled::collections::Iter;
use hydroflow::scheduled::handoff::VecHandoff;
use hydroflow::scheduled::{Hydroflow, SendCtx};
use hydroflow::{tl, tlt};

// const TRANSMISSIBLE_DURATION: Duration = Duration::from_secs(14 * 24 * 3600);
const TRANSMISSIBLE_DURATION: usize = 14;

fn main() {
    type Pid = &'static str;
    type Phone = &'static str;
    type DateTime = usize;

    let (contacts_send, contacts_recv) = mpsc::channel::<(Pid, Pid, DateTime)>();
    let (diagnosed_send, diagnosed_recv) = mpsc::channel::<(Pid, (DateTime, DateTime))>();
    let (people_send, people_recv) = mpsc::channel::<(Pid, Phone)>();

    let mut df = Hydroflow::new();

    let contacts_out = df.add_source(move |send: &mut SendCtx<VecHandoff<_>>| {
        send.give(Iter(contacts_recv.try_iter()));
    });
    let diagnosed_out = df.add_source(move |send: &mut SendCtx<VecHandoff<_>>| {
        send.give(Iter(diagnosed_recv.try_iter()));
    });
    let people_out = df.add_source(move |send: &mut SendCtx<VecHandoff<_>>| {
        send.give(Iter(people_recv.try_iter()));
    });

    // TODO[mingwei]: not the final API cause its bad.
    let (mut ltab, mut rtab, mut lbuf, mut rbuf) = Default::default();

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
                    .map(|(pid, t)| (pid, (t, t + TRANSMISSIBLE_DURATION)));

                let exposed = diagnosed_recv.take_inner().into_iter().chain(looped);

                let contacts = contacts_recv
                    .take_inner()
                    .into_iter()
                    .flat_map(|(pid_a, pid_b, t)| vec![(pid_a, (pid_b, t)), (pid_b, (pid_a, t))]);

                let join_exposed_contacts = SymmetricHashJoin::new(
                    exposed, contacts, &mut ltab, &mut rtab, &mut lbuf, &mut rbuf,
                );
                let new_exposed = join_exposed_contacts.filter_map(
                    |(_pid_a, (t_from, t_to), (pid_b, t_contact))| {
                        if t_from < t_contact && t_contact <= t_to {
                            // println!(
                            //     "DEBUG: post_join {} ({} {}) ({} {})",
                            //     _pid_a, t_from, t_to, pid_b, t_contact
                            // );
                            Some((pid_b, t_contact))
                        } else {
                            None
                        }
                    },
                );

                let notif_push = ForEach::new(|exposed_person: (Pid, DateTime)| {
                    // println!("DEBUG: will_notif {} {}", exposed_person.0, exposed_person.1);
                    notifs_send.give(Some(exposed_person));
                });
                let loop_push = ForEach::new(|exposed_person: (Pid, DateTime)| {
                    // println!("DEBUG: will_loop {}, {}", exposed_person.0, exposed_person.1);
                    loop_send.give(Some(exposed_person));
                });
                let push_exposed = Tee::new(notif_push, loop_push);

                let pivot = Pivot::new(new_exposed, push_exposed);
                pivot.run();
            },
        );

    df.add_edge(contacts_out, contacts_in);
    df.add_edge(diagnosed_out, diagnosed_in);
    df.add_edge(loop_out, loop_in);

    // TODO[mingwei]: not the final API cause its bad.
    let (mut ltab, mut rtab, mut lbuf, mut rbuf) = Default::default();

    type NotifsIn = tlt!(VecHandoff::<(Pid, Phone)>, VecHandoff::<(Pid, DateTime)>);
    let (tl!(people_in, notifs_in), ()) =
        df.add_subgraph::<_, NotifsIn, ()>(move |tl!(peoples, exposures), ()| {
            let exposures = exposures.take_inner().into_iter();
            let peoples = peoples.take_inner().into_iter();

            let joined = SymmetricHashJoin::new(
                peoples, exposures, &mut ltab, &mut rtab, &mut lbuf, &mut rbuf,
            );
            let joined_push = ForEach::new(|(pid, phone, exposure)| {
                println!("[{}] To {}: Possible Exposure at {}", pid, phone, exposure);
            });
            let pivot = Pivot::new(joined, joined_push);
            pivot.run();
        });

    df.add_edge(people_out, people_in);
    df.add_edge(notifs_out, notifs_in);

    df.run();

    people_send.send(("Mingwei S", "+1 650 555 7283")).unwrap();
    people_send.send(("Justin J", "+1 519 555 3458")).unwrap();
    people_send.send(("Mae M", "+1 912 555 9129")).unwrap();
    contacts_send
        .send((
            "Mingwei S",
            "Justin J",
            1031,
            // DateTime::new(
            //     Date::from_calendar_date(2021, Month::October, 31).unwrap(),
            //     Time::from_hms(19, 49, 21).unwrap(),
            // ),
        ))
        .unwrap();
    contacts_send
        .send((
            "Mingwei S",
            "Joe H",
            1027,
            // DateTime::new(
            //     Date::from_calendar_date(2021, Month::October, 27).unwrap(),
            //     Time::from_hms(16, 15, 00).unwrap(),
            // ),
        ))
        .unwrap();

    let mae_diag_datetime = 1022;
    // DateTime::new(
    //     Date::from_calendar_date(2021, Month::October, 22).unwrap(),
    //     Time::from_hms(15, 12, 55).unwrap(),
    // );
    diagnosed_send
        .send((
            "Mae M",
            (
                mae_diag_datetime,
                mae_diag_datetime + TRANSMISSIBLE_DURATION,
            ),
        ))
        .unwrap();

    df.run();

    contacts_send
        .send((
            "Mingwei S",
            "Mae M",
            1028,
            // DateTime::new(
            //     Date::from_calendar_date(2021, Month::October, 28).unwrap(),
            //     Time::from_hms(13, 56, 37).unwrap(),
            // ),
        ))
        .unwrap();

    df.run();
    df.run();
    df.run();
    df.run();

    println!("DONE");
}
