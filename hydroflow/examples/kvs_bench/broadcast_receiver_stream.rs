use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;
use tokio_stream::Stream;

#[derive(Debug)]
pub struct ReceiverStream<T: Clone> {
    inner: Receiver<T>,
}

impl<T: Clone> ReceiverStream<T> {
    pub fn new(recv: Receiver<T>) -> Self {
        Self { inner: recv }
    }
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
