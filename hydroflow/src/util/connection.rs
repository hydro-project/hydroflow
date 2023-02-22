use std::{path::PathBuf, pin::Pin};

use futures::{Stream, Sink};
use serde::{Deserialize, Serialize};
use tokio::net::{UnixListener, TcpListener, UnixStream, TcpStream};
use tokio_util::codec::LinesCodecError;

use super::{unix_lines, tcp_lines};

/// Describes a medium through which two HydroFlow services can communicate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
    InternalTcpPort(u16),
}

impl ConnectionPipe {
    pub async fn bind(&self) -> BoundConnection {
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
            ConnectionPipe::InternalTcpPort(port) => {
                BoundConnection::InternalTcpPort(TcpListener::bind(("127.0.0.1", *port)).await.unwrap())
            }
        }
    }

    pub async fn connect(&self) -> (
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
            ConnectionPipe::InternalTcpPort(port) => {
                let stream = TcpStream::connect(("127.0.0.1", *port)).await.unwrap();
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
    pub async fn accept_lines(&self) -> (
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
