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
pub enum ServerBindConfig {
    UnixSocket,
    TcpPort(
        /// The host the port should be bound on.
        String,
    ),
    Demux(HashMap<u32, ServerBindConfig>),
    Merge(Vec<ServerBindConfig>),
}

impl ServerBindConfig {
    #[async_recursion]
    pub async fn bind(self) -> BoundConnection {
        match self {
            ServerBindConfig::UnixSocket => {
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
            ServerBindConfig::TcpPort(host) => {
                let listener = TcpListener::bind((host, 0)).await.unwrap();
                BoundConnection::TcpPort(listener)
            }
            ServerBindConfig::Demux(bindings) => {
                let mut demux = HashMap::new();
                for (key, bind) in bindings {
                    demux.insert(key, bind.bind().await);
                }
                BoundConnection::Demux(demux)
            }
            ServerBindConfig::Merge(bindings) => {
                let mut merge = Vec::new();
                for bind in bindings {
                    merge.push(bind.bind().await);
                }
                BoundConnection::Merge(merge)
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

/// Describes how to connect to a service which is listening on some port.
#[allow(unreachable_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerPort {
    UnixSocket(PathBuf),
    TcpPort(SocketAddr),
    Demux(HashMap<u32, ServerPort>),
    Merge(Vec<ServerPort>),
}

pub type DynStream = Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send + Sync>>;

pub type DynSink<Input> = Pin<Box<dyn Sink<Input, Error = io::Error> + Send + Sync>>;

#[async_trait]
pub trait Connected: Send {
    async fn from_defn(pipe: ServerOrBound) -> Self;
}

pub trait ConnectedSink {
    type Input: Send;
    type Sink: Sink<Self::Input, Error = io::Error> + Send + Sync;

    fn take_sink(&mut self) -> Self::Sink;
}

pub trait ConnectedSource {
    type Output: Send;
    type Stream: Stream<Item = Result<Self::Output, io::Error>> + Send + Sync;
    fn take_source(&mut self) -> Self::Stream;
}

#[derive(Debug)]
pub enum BoundConnection {
    UnixSocket(UnixListener, tempfile::TempDir),
    TcpPort(TcpListener),
    Demux(HashMap<u32, BoundConnection>),
    Merge(Vec<BoundConnection>),
}

impl BoundConnection {
    pub fn sink_port(&self) -> ServerPort {
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
                    demux.insert(*key, bind.sink_port());
                }
                ServerPort::Demux(demux)
            }

            BoundConnection::Merge(bindings) => {
                let mut merge = Vec::new();
                for bind in bindings {
                    merge.push(bind.sink_port());
                }
                ServerPort::Merge(merge)
            }
        }
    }
}

#[async_recursion]
async fn accept(bound: &BoundConnection) -> (Option<DynStream>, Option<DynSink<Bytes>>) {
    match bound {
        BoundConnection::UnixSocket(listener, _) => {
            #[cfg(unix)]
            {
                let (stream, _) = listener.accept().await.unwrap();
                let (sink, source) = unix_bytes(stream);
                let out: (Option<DynStream>, Option<DynSink<Bytes>>) = (Some(Box::pin(source)), Some(Box::pin(sink)));
                out
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
            let out: (Option<DynStream>, Option<DynSink<Bytes>>) = (Some(Box::pin(source)), Some(Box::pin(sink)));
            out
        }
        BoundConnection::Merge(merge) => {
            let mut sources = vec![];
            for bound in merge {
                sources.push(accept(bound).await.0.unwrap());
            }

            let merge_source: DynStream = Box::pin(MergeSource {
                marker: PhantomData::default(),
                sources,
            });

            (Some(merge_source), None)
        },
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
                    source: bound.0,
                    sink: bound.1,
                }
            }
            ServerOrBound::Server(ServerPort::Merge(merge)) => {
                let sources = futures::future::join_all(merge.iter().map(|port| async {
                    ConnectedBidi::from_defn(ServerOrBound::Server(port.clone())).await.take_source()
                })).await;

                let merged = MergeSource {
                    marker: PhantomData::default(),
                    sources,
                };

                ConnectedBidi {
                    source: Some(Box::pin(merged)),
                    sink: None,
                }
            }
            ServerOrBound::Server(ServerPort::Demux(_)) => {
                panic!("Cannot connect to a demux pipe directly")
            }
        }
    }
}

impl ConnectedSource for ConnectedBidi {
    type Output = BytesMut;
    type Stream = DynStream;

    fn take_source(&mut self) -> DynStream {
        self.source.take().unwrap()
    }
}

impl ConnectedSink for ConnectedBidi {
    type Input = Bytes;
    type Sink = DynSink<Bytes>;

    fn take_sink(&mut self) -> DynSink<Self::Input> {
        self.sink.take().unwrap()
    }
}

pub struct ConnectedDemux<T: ConnectedSink> {
    pub keys: Vec<u32>,
    sink: Option<DemuxDrain<T::Input, T::Sink>>,
}

#[pin_project]
pub struct DemuxDrain<T, S: Sink<T, Error = io::Error> + Send + Sync + ?Sized> {
    marker: PhantomData<T>,
    #[pin]
    sinks: HashMap<u32, Pin<Box<S>>>,
}

impl<T, S: Sink<T, Error = io::Error> + Send + Sync> Sink<(u32, T)> for DemuxDrain<T, S> {
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
impl<T: Connected + ConnectedSink> Connected for ConnectedDemux<T>
where
    <T as ConnectedSink>::Input: 'static + Sync,
{
    async fn from_defn(pipe: ServerOrBound) -> Self {
        match pipe {
            ServerOrBound::Server(ServerPort::Demux(demux)) => {
                let mut connected_demux = HashMap::new();
                let keys = demux.keys().cloned().collect();
                for (id, pipe) in demux {
                    connected_demux.insert(
                        id,
                        Box::pin(T::from_defn(ServerOrBound::Server(pipe)).await.take_sink()),
                    );
                }

                let demuxer = DemuxDrain {
                    marker: PhantomData::default(),
                    sinks: connected_demux,
                };

                ConnectedDemux {
                    keys,
                    sink: Some(demuxer),
                }
            }

            ServerOrBound::Bound(BoundConnection::Demux(demux)) => {
                let mut connected_demux = HashMap::new();
                let keys = demux.keys().cloned().collect();
                for (id, bound) in demux {
                    connected_demux.insert(
                        id,
                        Box::pin(T::from_defn(ServerOrBound::Bound(bound)).await.take_sink()),
                    );
                }

                let demuxer = DemuxDrain {
                    marker: PhantomData::default(),
                    sinks: connected_demux,
                };

                ConnectedDemux {
                    keys,
                    sink: Some(demuxer),
                }
            }
            _ => panic!("Cannot connect to a non-demux pipe as a demux"),
        }
    }
}

impl<T: ConnectedSink> ConnectedSink for ConnectedDemux<T>
where
    <T as ConnectedSink>::Input: 'static + Sync,
{
    type Input = (u32, T::Input);
    type Sink = DemuxDrain<T::Input, T::Sink>;

    fn take_sink(&mut self) -> Self::Sink {
        self.sink.take().unwrap()
    }
}

#[pin_project]
pub struct MergeSource<T, S: Stream<Item = T> + Send + Sync + ?Sized> {
    marker: PhantomData<T>,
    #[pin]
    sources: Vec<Pin<Box<S>>>,
}

impl<T, S: Stream<Item = T> + Send + Sync + ?Sized> Stream for MergeSource<T, S> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let sources = &mut self.get_mut().sources;
        let mut next = None;

        let mut i = 0;
        while i < sources.len() {
            match sources[i].as_mut().poll_next(cx) {
                Poll::Ready(Some(v)) => {
                    next = Some(v);
                    break;
                }
                Poll::Ready(None) => {
                    // this happens infrequently, so OK to be O(n)
                    sources.remove(i);
                }
                Poll::Pending => {
                    i += 1;
                }
            }
        }

        if sources.is_empty() {
            Poll::Ready(None)
        } else if next.is_none() {
            Poll::Pending
        } else {
            Poll::Ready(next)
        }
    }
}
