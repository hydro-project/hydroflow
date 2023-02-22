use std::{path::PathBuf, pin::Pin};

use futures::{Sink, Stream};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio_util::codec::LinesCodecError;

use super::{tcp_lines, unix_lines};

/// Describes a medium through which two HydroFlow services can communicate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
    TcpPort(String, u16),
}

impl ConnectionPipe {
    pub async fn bind(self) -> BoundConnection {
        match self {
            ConnectionPipe::UnixSocket(path) => {
                #[cfg(unix)]
                {
                    BoundConnection::UnixSocket(UnixListener::bind(path).unwrap())
                }

                #[cfg(not(unix))]
                {
                    panic!("Unix sockets are not supported on this platform")
                }
            }
            ConnectionPipe::TcpPort(host, port) => {
                BoundConnection::InternalTcpPort(TcpListener::bind((host, port)).await.unwrap())
            }
        }
    }

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
            ConnectionPipe::TcpPort(host, port) => {
                let stream = TcpStream::connect((host, port)).await.unwrap();
                let (a, b) = tcp_lines(stream);
                (Box::pin(a), Box::pin(b))
            }
        }
    }
}

pub enum BoundConnection {
    UnixSocket(UnixListener),
    InternalTcpPort(TcpListener),
}

impl BoundConnection {
    pub async fn accept_lines(
        &self,
    ) -> (
        Pin<Box<dyn Sink<&'static str, Error = LinesCodecError>>>,
        Pin<Box<dyn Stream<Item = Result<String, LinesCodecError>>>>,
    ) {
        match self {
            BoundConnection::UnixSocket(listener) => {
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
            BoundConnection::InternalTcpPort(listener) => {
                let (stream, _) = listener.accept().await.unwrap();
                let (a, b) = tcp_lines(stream);
                (Box::pin(a), Box::pin(b))
            }
        }
    }
}
