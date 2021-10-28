use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Instant;
use hydroflow::scheduled::collections::Iter;
use hydroflow::scheduled::handoff::VecHandoff;
use hydroflow::scheduled::{Hydroflow, RecvCtx, SendCtx};

fn main() {
    type Pid = usize;
    type Phone = String;

    let (contacts_send, contacts_recv) = mpsc::channel::<(Pid, Pid)>();
    let (diagnosed_send, diagnosed_recv) = mpsc::channel::<(Pid, Instant, Instant)>();
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
}
