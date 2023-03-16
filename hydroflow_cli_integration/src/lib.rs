#![feature(never_type)]

use std::{
    collections::HashMap,
    marker::PhantomData,
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};

use futures::{ready, Sink, Stream};

use async_recursion::async_recursion;
use async_trait::async_trait;
use pin_project::pin_project;

use tokio::io;
use tokio::net::{TcpListener, TcpStream};

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[cfg(not(unix))]
#[allow(dead_code)]
type UnixStream = !;

#[cfg(not(unix))]
type UnixListener = !;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerConfig {
    UnixSocket,
    TcpPort(
        /// The host the port should be bound on.
        String,
    ),
    Demux(HashMap<u32, ServerConfig>),
}

impl ServerConfig {
    #[async_recursion]
    pub async fn bind(self) -> BoundConnection {
        match self {
            ServerConfig::UnixSocket => {
                #[cfg(unix)]
                {
                    let dir = tempfile::tempdir().unwrap();
                    let socket_path = dir.path().join("socket");
                    BoundConnection::UnixSocket(UnixListener::bind(socket_path).unwrap(), dir)
                }

                #[cfg(not(unix))]
                {
                    panic!("Unix sockets are not supported on this platform")
                }
            }
            ServerConfig::TcpPort(host) => {
                let listener = TcpListener::bind((host, 0)).await.unwrap();
                BoundConnection::TcpPort(listener)
            }
            ServerConfig::Demux(bindings) => {
                let mut demux = HashMap::new();
                for (key, bind) in bindings {
                    demux.insert(key, bind.bind().await);
                }
                BoundConnection::Demux(demux)
            }
        }
    }
}

pub enum ServerOrBound {
    Server(ServerPort),
    Bound(BoundConnection),
}

impl ServerOrBound {
    pub async fn connect<T: Connected>(self) -> T {
        T::from_defn(self).await
    }
}

/// Describes a medium through which two Hydroflow services can communicate.
#[allow(unreachable_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerPort {
    UnixSocket(PathBuf),
    TcpPort(SocketAddr),
    Demux(HashMap<u32, ServerPort>),
}

pub type DynStream = Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send>>;

pub type DynSink<Input> = Pin<Box<dyn Sink<Input, Error = io::Error> + Send + Sync>>;

#[async_trait]
pub trait Connected: Send {
    type Input: Send;

    fn take_source(&mut self) -> DynStream;
    fn take_sink(&mut self) -> DynSink<Self::Input>;

    async fn from_defn(pipe: ServerOrBound) -> Self;
}

#[derive(Debug)]
pub enum BoundConnection {
    UnixSocket(UnixListener, tempfile::TempDir),
    TcpPort(TcpListener),
    Demux(HashMap<u32, BoundConnection>),
}

impl BoundConnection {
    pub fn connection_defn(&self) -> ServerPort {
        match self {
            BoundConnection::UnixSocket(listener, _) => {
                #[cfg(unix)]
                {
                    ServerPort::UnixSocket(
                        listener
                            .local_addr()
                            .unwrap()
                            .as_pathname()
                            .unwrap()
                            .to_path_buf(),
                    )
                }

                #[cfg(not(unix))]
                {
                    let _ = listener;
                    panic!("Unix sockets are not supported on this platform")
                }
            }
            BoundConnection::TcpPort(listener) => {
                let addr = listener.local_addr().unwrap();
                ServerPort::TcpPort(SocketAddr::new(addr.ip(), addr.port()))
            }

            BoundConnection::Demux(bindings) => {
                let mut demux = HashMap::new();
                for (key, bind) in bindings {
                    demux.insert(*key, bind.connection_defn());
                }
                ServerPort::Demux(demux)
            }
        }
    }
}

async fn accept(bound: &BoundConnection) -> (DynStream, DynSink<Bytes>) {
    match bound {
        BoundConnection::UnixSocket(listener, _) => {
            #[cfg(unix)]
            {
                let (stream, _) = listener.accept().await.unwrap();
                let (sink, source) = unix_bytes(stream);
                (Box::pin(source), Box::pin(sink))
            }

            #[cfg(not(unix))]
            {
                let _ = listener;
                panic!("Unix sockets are not supported on this platform")
            }
        }
        BoundConnection::TcpPort(listener) => {
            let (stream, _) = listener.accept().await.unwrap();
            let (sink, source) = tcp_bytes(stream);
            (Box::pin(source), Box::pin(sink))
        }
        BoundConnection::Demux(_) => panic!("Cannot connect to a demux pipe directly"),
    }
}

fn tcp_bytes(
    stream: TcpStream,
) -> (
    FramedWrite<tokio::net::tcp::OwnedWriteHalf, LengthDelimitedCodec>,
    FramedRead<tokio::net::tcp::OwnedReadHalf, LengthDelimitedCodec>,
) {
    let (recv, send) = stream.into_split();
    let send = FramedWrite::new(send, LengthDelimitedCodec::new());
    let recv = FramedRead::new(recv, LengthDelimitedCodec::new());
    (send, recv)
}

#[cfg(unix)]
fn unix_bytes(
    stream: UnixStream,
) -> (
    FramedWrite<tokio::net::unix::OwnedWriteHalf, LengthDelimitedCodec>,
    FramedRead<tokio::net::unix::OwnedReadHalf, LengthDelimitedCodec>,
) {
    let (recv, send) = stream.into_split();
    let send = FramedWrite::new(send, LengthDelimitedCodec::new());
    let recv = FramedRead::new(recv, LengthDelimitedCodec::new());
    (send, recv)
}

pub struct ConnectedBidi {
    source: Option<DynStream>,
    sink: Option<DynSink<Bytes>>,
}

#[async_trait]
impl Connected for ConnectedBidi {
    type Input = Bytes;

    fn take_source(&mut self) -> DynStream {
        self.source.take().unwrap()
    }

    fn take_sink(&mut self) -> DynSink<Self::Input> {
        self.sink.take().unwrap()
    }

    async fn from_defn(pipe: ServerOrBound) -> Self {
        match pipe {
            ServerOrBound::Server(ServerPort::UnixSocket(path)) => {
                #[cfg(unix)]
                {
                    let stream = UnixStream::connect(path).await.unwrap();
                    let (sink, source) = unix_bytes(stream);
                    ConnectedBidi {
                        source: Some(Box::pin(source)),
                        sink: Some(Box::pin(sink)),
                    }
                }

                #[cfg(not(unix))]
                {
                    let _ = path;
                    panic!("Unix sockets are not supported on this platform");
                }
            }
            ServerOrBound::Server(ServerPort::TcpPort(addr)) => {
                let stream = TcpStream::connect(addr).await.unwrap();
                let (sink, source) = tcp_bytes(stream);
                ConnectedBidi {
                    source: Some(Box::pin(source)),
                    sink: Some(Box::pin(sink)),
                }
            }
            ServerOrBound::Bound(bound) => {
                let bound = accept(&bound).await;
                ConnectedBidi {
                    source: Some(bound.0),
                    sink: Some(bound.1),
                }
            }
            ServerOrBound::Server(ServerPort::Demux(_)) => {
                panic!("Cannot connect to a demux pipe directly")
            }
        }
    }
}

pub struct ConnectedDemux<T: Connected> {
    pub keys: Vec<u32>,
    sink: Option<DynSink<(u32, T::Input)>>,
}

#[pin_project]
struct DemuxDrain<T> {
    marker: PhantomData<T>,
    #[pin]
    sinks: HashMap<u32, DynSink<T>>,
}

impl<T> Sink<(u32, T)> for DemuxDrain<T> {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for sink in self.project().sinks.values_mut() {
            ready!(Sink::poll_ready(sink.as_mut(), _cx))?;
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: (u32, T)) -> Result<(), Self::Error> {
        Sink::start_send(
            self.project()
                .sinks
                .get_mut()
                .get_mut(&item.0)
                .unwrap()
                .as_mut(),
            item.1,
        )
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for sink in self.project().sinks.values_mut() {
            ready!(Sink::poll_flush(sink.as_mut(), _cx))?;
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for sink in self.project().sinks.values_mut() {
            ready!(Sink::poll_close(sink.as_mut(), _cx))?;
        }

        Poll::Ready(Ok(()))
    }
}

#[async_trait]
impl<T: Connected> Connected for ConnectedDemux<T>
where
    <T as Connected>::Input: 'static + Sync,
{
    type Input = (u32, T::Input);

    fn take_source(&mut self) -> DynStream {
        panic!("Cannot take source from a demux pipe");
    }

    fn take_sink(&mut self) -> DynSink<Self::Input> {
        self.sink.take().unwrap()
    }

    async fn from_defn(pipe: ServerOrBound) -> Self {
        match pipe {
            ServerOrBound::Server(ServerPort::Demux(demux)) => {
                let mut connected_demux = HashMap::new();
                let keys = demux.keys().cloned().collect();
                for (id, pipe) in demux {
                    connected_demux.insert(
                        id,
                        T::from_defn(ServerOrBound::Server(pipe)).await.take_sink(),
                    );
                }

                let demuxer = DemuxDrain {
                    marker: PhantomData::default(),
                    sinks: connected_demux,
                };

                ConnectedDemux {
                    keys,
                    sink: Some(Box::pin(demuxer)),
                }
            }

            ServerOrBound::Bound(bound) => {
                if let BoundConnection::Demux(demux) = bound {
                    let mut connected_demux = HashMap::new();
                    let keys = demux.keys().cloned().collect();
                    for (id, bound) in demux {
                        connected_demux.insert(
                            id,
                            T::from_defn(ServerOrBound::Bound(bound)).await.take_sink(),
                        );
                    }

                    let demuxer = DemuxDrain {
                        marker: PhantomData::default(),
                        sinks: connected_demux,
                    };

                    ConnectedDemux {
                        keys,
                        sink: Some(Box::pin(demuxer)),
                    }
                } else {
                    panic!("Cannot connect to a non-demux pipe as a demux")
                }
            }
            _ => panic!("Cannot connect to a non-demux pipe as a demux"),
        }
    }
}
