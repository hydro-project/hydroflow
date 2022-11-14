//! Helper utilities for the Hydroflow surface syntax.

use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_util::codec::{Decoder, Encoder, LinesCodec};
use tokio_util::udp::UdpFramed;

pub fn unbounded_channel<T>() -> (
    tokio::sync::mpsc::UnboundedSender<T>,
    tokio_stream::wrappers::UnboundedReceiverStream<T>,
) {
    let (send, recv) = tokio::sync::mpsc::unbounded_channel();
    let recv = tokio_stream::wrappers::UnboundedReceiverStream::new(recv);
    (send, recv)
}

pub type UdpFramedSink<Codec, Item> = SplitSink<UdpFramed<Codec>, (Item, SocketAddr)>;
pub type UdpFramedStream<Codec> = SplitStream<UdpFramed<Codec>>;

pub fn udp_framed<Codec, Item>(
    socket: UdpSocket,
    codec: Codec,
) -> (UdpFramedSink<Codec, Item>, UdpFramedStream<Codec>)
where
    Codec: Encoder<Item> + Decoder,
{
    let framed = UdpFramed::new(socket, codec);
    futures::stream::StreamExt::split(framed)
}

pub fn udp_bytes(
    socket: UdpSocket,
) -> (
    UdpFramedSink<LengthDelimitedCodec, Bytes>,
    UdpFramedStream<LengthDelimitedCodec>,
) {
    udp_framed(socket, LengthDelimitedCodec::new())
}

pub fn udp_lines(
    socket: UdpSocket,
) -> (
    UdpFramedSink<LinesCodec, String>,
    UdpFramedStream<LinesCodec>,
) {
    udp_framed(socket, LinesCodec::new())
}

pin_project! {
    pub struct CollectReady<St, Out> {
        #[pin]
        stream: St,
        _phantom: PhantomData<Out>,
    }
}
impl<St, Out> Future for CollectReady<St, Out>
where
    St: Stream,
    Out: FromIterator<St::Item>,
{
    type Output = Out;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let out = std::iter::from_fn(move || match Pin::new(&mut this.stream).poll_next(ctx) {
            Poll::Pending => None,
            Poll::Ready(opt_item) => opt_item,
        })
        .collect();
        Poll::Ready(out)
    }
}

/// Collects the immediately available items into the stream.
///
/// This consumes the stream, use [`futures::StreamExt::by_ref()`] if you want
/// to retain ownership of your stream.
pub async fn collect_ready<St, Out>(stream: St) -> Out
where
    St: Stream,
    Out: FromIterator<St::Item>,
{
    let collect_ready = CollectReady {
        stream,
        _phantom: PhantomData,
    };
    collect_ready.await
}

/// Receives available items in an `UnboundedReceiverStream` into a `FromIterator` collection.
pub fn recv_into<C, T>(recv: &mut UnboundedReceiverStream<T>) -> C
where
    C: FromIterator<T>,
{
    std::iter::from_fn(|| recv.as_mut().try_recv().ok()).collect()
}
