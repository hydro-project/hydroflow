use std::collections::HashMap;

use futures::channel::mpsc::{channel, Receiver, Sender};
use hydroflow::lang::{
    lattice::{dom_pair::DomPairRepr, map_union::MapUnionRepr, ord::MaxRepr},
    tag,
};

pub(crate) type Clock = HashMap<usize, u64>;

pub(crate) fn spawn<F, K, V>(n: u64, f: F) -> Vec<Sender<Message<K, V>>>
where
    F: 'static + Fn(usize, Receiver<Message<K, V>>, Vec<Sender<Message<K, V>>>) + Send + Clone,
    K: 'static + Send + Clone,
    V: 'static + Ord + Send + Clone,
{
    let (matrix, senders) = make_communication_matrix(n);
    for (i, (receiver, senders)) in matrix.into_iter().enumerate() {
        let f = f.clone();
        std::thread::spawn(move || f(i, receiver, senders));
    }

    senders
}

#[derive(Debug)]
pub(crate) enum Message<K, V>
where
    K: Send + Clone,
    V: Send + Clone,
{
    // A KV set request from a client.
    Set(K, V),
    // A KV get request from a client.
    Get(K, futures::channel::oneshot::Sender<(Clock, V)>),
    // A set of data that I am responsible for, sent to me by another worker.
    Batch((usize, u64), Vec<(K, V)>),
}

type Matrix<K, V> = Vec<(Receiver<Message<K, V>>, Vec<Sender<Message<K, V>>>)>;
type MessageSender<K, V> = Sender<Message<K, V>>;

fn make_communication_matrix<K, V>(n: u64) -> (Matrix<K, V>, Vec<MessageSender<K, V>>)
where
    K: Send + Clone,
    V: Ord + Send + Clone,
{
    let mut receivers = Vec::new();
    let mut senders: Vec<_> = (0..n).map(|_| Vec::new()).collect();
    let mut extra_senders = Vec::new();
    for _ in 0..n {
        let (sender, receiver) = channel(8192);
        receivers.push(receiver);
        for s in senders.iter_mut() {
            s.push(sender.clone())
        }
        extra_senders.push(sender);
    }

    (
        receivers.into_iter().zip(senders.into_iter()).collect(),
        extra_senders,
    )
}

pub(crate) type ClockRepr = MapUnionRepr<tag::HASH_MAP, usize, MaxRepr<u64>>;
pub(crate) type ClockUpdateRepr = MapUnionRepr<tag::SINGLE, usize, MaxRepr<u64>>;

pub(crate) type ClockedDataRepr<V> = DomPairRepr<ClockRepr, MaxRepr<V>>;
pub(crate) type ClockedUpdateRepr<V> = DomPairRepr<ClockUpdateRepr, MaxRepr<V>>;
