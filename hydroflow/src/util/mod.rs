//! Helper utilities for the Hydroflow surface syntax.

use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use bincode;
use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use futures::Stream;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_util::codec::{Decoder, Encoder, LinesCodec, LinesCodecError};
use tokio_util::udp::UdpFramed;

pub type UdpFramedSink<Codec, Item> = SplitSink<UdpFramed<Codec>, (Item, SocketAddr)>;
pub type UdpFramedStream<Codec> = SplitStream<UdpFramed<Codec>>;
pub type UdpSink = UdpFramedSink<LengthDelimitedCodec, Bytes>;
pub type UdpStream = UdpFramedStream<LengthDelimitedCodec>;

pub fn unbounded_channel<T>() -> (
    tokio::sync::mpsc::UnboundedSender<T>,
    tokio_stream::wrappers::UnboundedReceiverStream<T>,
) {
    let (send, recv) = tokio::sync::mpsc::unbounded_channel();
    let recv = tokio_stream::wrappers::UnboundedReceiverStream::new(recv);
    (send, recv)
}

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

pub fn udp_bytes(socket: UdpSocket) -> (UdpSink, UdpStream) {
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

pub fn serialize_msg<T>(msg: T) -> bytes::Bytes
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    bytes::Bytes::from(bincode::serialize(&msg).unwrap())
}

pub fn deserialize_simple<T>(msg: bytes::BytesMut) -> T
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    bincode::deserialize(&msg).unwrap()
}

pub fn deserialize_msg<T>(msg: Result<(bytes::BytesMut, SocketAddr), LinesCodecError>) -> T
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    bincode::deserialize(&(msg.unwrap().0)).unwrap()
}

pub fn ipv4_resolve(addr: String) -> SocketAddr {
    use std::net::ToSocketAddrs;
    let mut addrs = addr.to_socket_addrs().unwrap();
    addrs
        .find(|addr| addr.is_ipv4())
        .expect("Unable to resolve connection address")
}

pub async fn bind_udp_socket_addr(addr: SocketAddr) -> (UdpSink, UdpStream) {
    let socket = tokio::net::UdpSocket::bind(addr).await.unwrap();
    udp_bytes(socket)
}

pub async fn bind_udp_socket(addr_string: String) -> (UdpSink, UdpStream) {
    let addr = ipv4_resolve(addr_string);
    bind_udp_socket_addr(addr).await
}

pub async fn bind_local_udp_socket(port: u16) -> (UdpSink, UdpStream) {
    bind_udp_socket(format!("127.0.0.1:{}", port)).await
}
