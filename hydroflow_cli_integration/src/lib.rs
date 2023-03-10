use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "runtime")]
use std::{pin::Pin, sync::Arc};

#[cfg(feature = "runtime")]
use async_trait::async_trait;
#[cfg(feature = "runtime")]
use bytes::BytesMut;
#[cfg(feature = "runtime")]
use futures::{Sink, Stream};

#[cfg(feature = "runtime")]
#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(feature = "runtime")]
#[cfg(not(unix))]
type UnixListener = !;

#[cfg(feature = "runtime")]
use tokio::{io, net::TcpListener};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BindConfig {
    UnixSocket,
    TcpPort(
        /// The host the port should be bound on.
        String,
    ),
}

impl BindConfig {
    #[cfg(feature = "runtime")]
    pub async fn bind(self) -> BoundConnection {
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

/// Describes a medium through which two Hydroflow services can communicate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
    TcpPort(SocketAddr),
    Demux(HashMap<u32, ConnectionPipe>),
    #[cfg(feature = "runtime")]
    #[serde(skip)]
    Bind(Arc<BoundConnection>),
}

#[cfg(feature = "runtime")]
impl ConnectionPipe {
    pub async fn connect<T: Connected>(self) -> T {
        T::from_pipe(self).await
    }
}

#[cfg(feature = "runtime")]
pub type DynStream = Pin<Box<dyn Stream<Item = Result<BytesMut, io::Error>> + Send>>;
#[cfg(feature = "runtime")]
pub type DynSink<Input> = Pin<Box<dyn Sink<Input, Error = io::Error> + Send + Sync>>;

#[cfg(feature = "runtime")]
#[async_trait]
pub trait Connected: Send {
    type Input: Send;

    fn take_source(&mut self) -> DynStream;
    fn take_sink(&mut self) -> DynSink<Self::Input>;

    async fn from_pipe(pipe: ConnectionPipe) -> Self;
}

#[cfg(feature = "runtime")]
#[derive(Debug)]
pub enum BoundConnection {
    UnixSocket(UnixListener, tempfile::TempDir),
    TcpPort(TcpListener),
}

#[cfg(feature = "runtime")]
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
}
