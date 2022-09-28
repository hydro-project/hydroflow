//! Helper utilities for the Hydroflow surface syntax.

use std::net::SocketAddr;

use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use tokio::net::UdpSocket;
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
