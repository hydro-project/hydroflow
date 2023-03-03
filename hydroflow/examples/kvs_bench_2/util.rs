use crate::broadcast_receiver_stream::ReceiverStream;
use tokio::sync::broadcast::{channel, Sender};

pub fn bounded_broadcast_channel<T: Clone>(capacity: usize) -> (Sender<T>, ReceiverStream<T>) {
    let (send, recv) = channel(capacity);
    let recv = ReceiverStream::new(recv);
    (send, recv)
}
