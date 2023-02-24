use std::{net::SocketAddr, path::PathBuf, pin::Pin};

use futures::{Sink, Stream};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio_util::codec::LinesCodecError;

use super::{tcp_lines, unix_lines};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BindType {
    UnixSocket,
    TcpPort(
        /// The host the port should be bound on.
        String,
    ),
}

impl BindType {
    pub async fn bind(self) -> BoundConnection {
        match self {
            BindType::UnixSocket => {
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
            BindType::TcpPort(host) => {
                let listener = TcpListener::bind((host, 0)).await.unwrap();
                BoundConnection::TcpPort(listener)
            }
        }
    }
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
        Pin<Box<dyn Sink<&'static str, Error = LinesCodecError> + Send>>,
        Pin<Box<dyn Stream<Item = Result<String, LinesCodecError>>>>,
    ) {
        match self {
            ConnectionPipe::UnixSocket(path) => {
                #[cfg(unix)]
                {
                    let stream = UnixStream::connect(path).await.unwrap();
                    let (a, b) = unix_lines(stream);
                    (Box::pin(a), Box::pin(b))
                }

                #[cfg(not(unix))]
                {
                    panic!("Unix sockets are not supported on this platform")
                }
            }
            ConnectionPipe::TcpPort(addr) => {
                let stream = TcpStream::connect(addr).await.unwrap();
                let (a, b) = tcp_lines(stream);
                (Box::pin(a), Box::pin(b))
            }
        }
    }
}

pub enum BoundConnection {
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
                    panic!("Unix sockets are not supported on this platform")
                }
            }
            BoundConnection::TcpPort(listener) => {
                let addr = listener.local_addr().unwrap();
                ConnectionPipe::TcpPort(SocketAddr::new(addr.ip(), addr.port()))
            }
        }
    }

    pub async fn accept_lines(
        &self,
    ) -> (
        Pin<Box<dyn Sink<&'static str, Error = LinesCodecError>>>,
        Pin<Box<dyn Stream<Item = Result<String, LinesCodecError>>>>,
    ) {
        match self {
            BoundConnection::UnixSocket(listener, _) => {
                #[cfg(unix)]
                {
                    let (stream, _) = listener.accept().await.unwrap();
                    let (a, b) = unix_lines(stream);
                    (Box::pin(a), Box::pin(b))
                }

                #[cfg(not(unix))]
                {
                    panic!("Unix sockets are not supported on this platform")
                }
            }
            BoundConnection::TcpPort(listener) => {
                let (stream, _) = listener.accept().await.unwrap();
                let (a, b) = tcp_lines(stream);
                (Box::pin(a), Box::pin(b))
            }
        }
    }
}
