use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;
use tokio_stream::Stream;

/// A wrapper around [`tokio::sync::broadcast::Receiver`] that implements [`Stream`].
///
/// [`tokio::sync::broadcast::Receiver`]: struct@tokio::sync::broadcast::Receiver
/// [`Stream`]: trait@crate::Stream
#[derive(Debug)]
pub struct ReceiverStream<T: Clone> {
    inner: Receiver<T>,
}

impl<T: Clone> ReceiverStream<T> {
    /// Create a new `UnboundedReceiverStream`.
    pub fn new(recv: Receiver<T>) -> Self {
        Self { inner: recv }
    }

    // /// Get back the inner `UnboundedReceiver`.
    // pub fn into_inner(self) -> Receiver<T> {
    //     self.inner
    // }

    // /// Closes the receiving half of a channel without dropping it.
    // ///
    // /// This prevents any further messages from being sent on the channel while
    // /// still enabling the receiver to drain messages that are buffered.
    // pub fn close(&mut self) {
    //     self.inner.close()
    // }
}

impl<T: Clone> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.try_recv() {
            Ok(x) => Poll::Ready(Some(x)),
            Err(e) => match e {
                TryRecvError::Empty => Poll::Pending,
                TryRecvError::Closed => Poll::Ready(None),
                TryRecvError::Lagged(_) => panic!(),
            },
        }
    }
}

impl<T: Clone> AsRef<Receiver<T>> for ReceiverStream<T> {
    fn as_ref(&self) -> &Receiver<T> {
        &self.inner
    }
}

impl<T: Clone> AsMut<Receiver<T>> for ReceiverStream<T> {
    fn as_mut(&mut self) -> &mut Receiver<T> {
        &mut self.inner
    }
}

impl<T: Clone> From<Receiver<T>> for ReceiverStream<T> {
    fn from(recv: Receiver<T>) -> Self {
        Self::new(recv)
    }
}

impl<T: Clone> Clone for ReceiverStream<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.resubscribe(),
        }
    }
}
