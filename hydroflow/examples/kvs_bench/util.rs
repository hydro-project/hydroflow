use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::BroadcastStream;

pub fn bounded_broadcast_channel<T: 'static + Clone + Send>(
    capacity: usize,
) -> (Sender<T>, BroadcastStream<T>) {
    let (send, recv) = channel(capacity);
    let recv = BroadcastStream::new(recv);
    (send, recv)
}
