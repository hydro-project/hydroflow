use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::{
    BytesCodec, Decoder, FramedRead, FramedWrite, LengthDelimitedCodec, LinesCodec,
};

/// Helper creates a TCP `Stream` and `Sink` from the given socket, using the given `Codec` to
/// handle delineation between inputs/outputs.
pub fn tcp_framed<Codec>(
    stream: TcpStream,
    codec: Codec,
) -> (
    FramedWrite<OwnedWriteHalf, Codec>,
    FramedRead<OwnedReadHalf, Codec>,
)
where
    Codec: Clone + Decoder,
{
    let (recv, send) = stream.into_split();
    let send = FramedWrite::new(send, codec.clone());
    let recv = FramedRead::new(recv, codec);
    (send, recv)
}

/// Helper creates a TCP `Stream` and `Sink` for `Bytes` strings where each string is
/// length-delimited.
pub fn tcp_bytes(
    stream: TcpStream,
) -> (
    FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
) {
    tcp_framed(stream, LengthDelimitedCodec::new())
}

/// Helper creates a TCP `Stream` and `Sink` for undelimited streams of `Bytes`.
pub fn tcp_bytestream(
    stream: TcpStream,
) -> (
    FramedWrite<OwnedWriteHalf, BytesCodec>,
    FramedRead<OwnedReadHalf, BytesCodec>,
) {
    tcp_framed(stream, BytesCodec::new())
}

/// Helper creates a TCP `Stream` and `Sink` for `str`ings delimited by newlines.
pub fn tcp_lines(
    stream: TcpStream,
) -> (
    FramedWrite<OwnedWriteHalf, LinesCodec>,
    FramedRead<OwnedReadHalf, LinesCodec>,
) {
    tcp_framed(stream, LinesCodec::new())
}
