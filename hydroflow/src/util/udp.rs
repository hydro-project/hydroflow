use std::net::SocketAddr;

use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use tokio::net::UdpSocket;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_util::codec::{BytesCodec, Decoder, Encoder, LinesCodec};
use tokio_util::udp::UdpFramed;

pub type UdpFramedSink<Codec, Item> = SplitSink<UdpFramed<Codec>, (Item, SocketAddr)>;
pub type UdpFramedStream<Codec> = SplitStream<UdpFramed<Codec>>;
/// Helper creates a UDP `Stream` and `Sink` from the given socket, using the given `Codec` to
/// handle delineation between inputs/outputs.
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

pub type UdpSink = UdpFramedSink<LengthDelimitedCodec, Bytes>;
pub type UdpStream = UdpFramedStream<LengthDelimitedCodec>;
/// Helper creates a UDP `Stream` and `Sink` for `Bytes` strings where each string is
/// length-delimited.
pub fn udp_bytes(socket: UdpSocket) -> (UdpSink, UdpStream) {
    udp_framed(socket, LengthDelimitedCodec::new())
}

pub type UdpBytesSink = UdpFramedSink<BytesCodec, Bytes>;
pub type UdpBytesStream = UdpFramedStream<BytesCodec>;
/// Helper creates a UDP `Stream` and `Sink` for undelimited streams of `Bytes`.
pub fn udp_bytestream(socket: UdpSocket) -> (UdpBytesSink, UdpBytesStream) {
    udp_framed(socket, BytesCodec::new())
}

pub type UdpLinesSink = UdpFramedSink<LinesCodec, String>;
pub type UdpLinesStream = UdpFramedStream<LinesCodec>;
/// Helper creates a UDP `Stream` and `Sink` for `str`ings delimited by newlines.
pub fn udp_lines(
    socket: UdpSocket,
) -> (
    UdpFramedSink<LinesCodec, String>,
    UdpFramedStream<LinesCodec>,
) {
    udp_framed(socket, LinesCodec::new())
}
