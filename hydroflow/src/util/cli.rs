use std::{
    collections::HashMap,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use bytes::Bytes;
use futures::{Sink, SinkExt};
use hydroflow_cli_integration::{BindConfig, BoundConnection, ConnectionPipe, DynSink, DynStream};
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::{io, net::TcpStream, sync::Mutex};

use super::tcp_bytes;
#[cfg(unix)]
use super::unix_bytes;

pub use hydroflow_cli_integration::Connected;

async fn accept_incoming_connections(
    binds: HashMap<String, BoundConnection>,
) -> HashMap<String, ConnectionPipe> {
    let mut bind_results = HashMap::new();
    for (name, bind) in binds {
        bind_results.insert(name, ConnectionPipe::Bind(bind));
    }
    bind_results
}

pub async fn init() -> HashMap<String, ConnectionPipe> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let bind_config = serde_json::from_str::<HashMap<String, BindConfig>>(trimmed).unwrap();

    // bind to sockets
    let mut bind_results: HashMap<String, ConnectionPipe> = HashMap::new();
    let mut binds = HashMap::new();
    for (name, config) in bind_config {
        let bound = config.bind().await;
        bind_results.insert(name.clone(), bound.connection_pipe());
        binds.insert(name.clone(), bound);
    }

    let bind_connected_future =
        tokio::task::spawn(async move { accept_incoming_connections(binds).await });

    let bind_serialized = serde_json::to_string(&bind_results).unwrap();
    println!("ready: {bind_serialized}");

    let mut start_buf = String::new();
    std::io::stdin().read_line(&mut start_buf).unwrap();
    let connection_pipes = if start_buf.starts_with("start: ") {
        serde_json::from_str::<HashMap<String, ConnectionPipe>>(
            start_buf.trim_start_matches("start: ").trim(),
        )
        .unwrap()
    } else {
        panic!("expected start");
    };

    let mut all_connected = HashMap::new();
    for (name, pipe) in connection_pipes {
        all_connected.insert(name, pipe);
    }

    let bind_connected = bind_connected_future.await.unwrap();
    for (name, pipe) in bind_connected {
        all_connected.insert(name, pipe);
    }

    all_connected
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

    async fn from_pipe(pipe: ConnectionPipe) -> Self {
        match pipe {
            ConnectionPipe::UnixSocket(path) => {
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
            ConnectionPipe::TcpPort(addr) => {
                let stream = TcpStream::connect(addr).await.unwrap();
                let (sink, source) = tcp_bytes(stream);
                ConnectedBidi {
                    source: Some(Box::pin(source)),
                    sink: Some(Box::pin(sink)),
                }
            }
            ConnectionPipe::Bind(bound) => {
                let bound = accept(&bound).await;
                ConnectedBidi {
                    source: Some(bound.0),
                    sink: Some(bound.1),
                }
            }
            ConnectionPipe::Demux(_) => panic!("Cannot connect to a demux pipe directly"),
        }
    }
}

pub struct ConnectedDemux<T: Connected> {
    pub keys: Vec<u32>,
    sink: Option<DynSink<(u32, T::Input)>>,
}

// a copy of `Drain` that uses `io::Error` instead of `Infallible`
struct IoErrorDrain<T> {
    marker: PhantomData<T>,
}

impl<T> Sink<T> for IoErrorDrain<T> {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, _item: T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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

    async fn from_pipe(pipe: ConnectionPipe) -> Self {
        match pipe {
            ConnectionPipe::Demux(demux) => {
                let mut connected_demux = HashMap::new();
                let keys = demux.keys().cloned().collect();
                for (id, pipe) in demux {
                    connected_demux.insert(
                        id,
                        Arc::new(Mutex::new(T::from_pipe(pipe).await.take_sink())),
                    );
                }

                let demuxer = IoErrorDrain {
                    marker: PhantomData::default(),
                }
                .with(move |(id, input)| {
                    let sink = connected_demux.get_mut(&id).unwrap().clone();
                    async move { sink.lock().await.feed(input).await }
                });

                ConnectedDemux {
                    keys,
                    sink: Some(Box::pin(demuxer)),
                }
            }

            ConnectionPipe::Bind(bound) => {
                if let BoundConnection::Demux(demux) = bound {
                    let mut connected_demux = HashMap::new();
                    let keys = demux.keys().cloned().collect();
                    for (id, bound) in demux {
                        connected_demux.insert(
                            id,
                            Arc::new(Mutex::new(
                                T::from_pipe(ConnectionPipe::Bind(bound)).await.take_sink(),
                            )),
                        );
                    }

                    let demuxer = IoErrorDrain {
                        marker: PhantomData::default(),
                    }
                    .with(move |(id, input)| {
                        let sink = connected_demux.get_mut(&id).unwrap().clone();
                        async move { sink.lock().await.feed(input).await }
                    });

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
