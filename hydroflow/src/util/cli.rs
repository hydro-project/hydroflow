use std::{collections::HashMap, net::SocketAddr, path::PathBuf, pin::Pin};

use bytes::{Bytes, BytesMut};
use futures::{Sink, Stream};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
#[cfg(not(unix))]
type UnixListener = !;

use super::tcp_bytes;
#[cfg(unix)]
use super::unix_bytes;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BindConfig {
    UnixSocket,
    TcpPort(
        /// The host the port should be bound on.
        String,
    ),
}

impl BindConfig {
    async fn bind(self) -> BoundConnection {
        match self {
            BindConfig::UnixSocket => {
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
            BindConfig::TcpPort(host) => {
                let listener = TcpListener::bind((host, 0)).await.unwrap();
                BoundConnection::TcpPort(listener)
            }
        }
    }
}

async fn accept_incoming_connections(
    binds: HashMap<String, BoundConnection>,
) -> HashMap<
    String,
    (
        Option<Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send>>>,
        Option<Pin<Box<dyn Sink<Bytes, Error = io::Error> + Send>>>,
    ),
> {
    let mut bind_results = HashMap::new();
    for (name, bind) in binds {
        let bound = bind.accept().await;
        bind_results.insert(name.clone(), (Some(bound.0), Some(bound.1)));
    }
    bind_results
}

pub async fn init() -> HashMap<
    String,
    (
        Option<Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send>>>,
        Option<Pin<Box<dyn Sink<Bytes, Error = io::Error> + Send>>>,
    ),
> {
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
        let (sink, source) = pipe.connect().await;
        all_connected.insert(name, (sink, source));
    }

    let bind_connected = bind_connected_future.await.unwrap();
    for (name, (sink, source)) in bind_connected {
        all_connected.insert(name, (sink, source));
    }

    all_connected
}

/// Describes a medium through which two HydroFlow services can communicate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
    TcpPort(SocketAddr),
}

impl ConnectionPipe {
    pub async fn connect(
        self,
    ) -> (
        Option<Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send>>>,
        Option<Pin<Box<dyn Sink<Bytes, Error = io::Error> + Send>>>,
    ) {
        match self {
            ConnectionPipe::UnixSocket(path) => {
                #[cfg(unix)]
                {
                    let stream = UnixStream::connect(path).await.unwrap();
                    let (sink, source) = unix_bytes(stream);
                    (Some(Box::pin(source)), Some(Box::pin(sink)))
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
                (Some(Box::pin(source)), Some(Box::pin(sink)))
            }
        }
    }
}

enum BoundConnection {
    UnixSocket(UnixListener, tempfile::TempDir),
    TcpPort(TcpListener),
}

impl BoundConnection {
    pub fn connection_pipe(&self) -> ConnectionPipe {
        match self {
            BoundConnection::UnixSocket(listener, _) => {
                #[cfg(unix)]
                {
                    ConnectionPipe::UnixSocket(
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
                ConnectionPipe::TcpPort(SocketAddr::new(addr.ip(), addr.port()))
            }
        }
    }

    pub async fn accept(
        &self,
    ) -> (
        Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send>>,
        Pin<Box<dyn Sink<Bytes, Error = io::Error> + Send>>,
    ) {
        match self {
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
        }
    }
}
