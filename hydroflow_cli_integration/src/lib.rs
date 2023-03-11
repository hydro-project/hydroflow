use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "runtime")]
use std::pin::Pin;

#[cfg(feature = "runtime")]
use async_trait::async_trait;
#[cfg(feature = "runtime")]
use bytes::BytesMut;
#[cfg(feature = "runtime")]
use futures::{Sink, Stream};

#[cfg(feature = "runtime")]
use async_recursion::async_recursion;

#[cfg(feature = "runtime")]
#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(feature = "runtime")]
#[cfg(not(unix))]
type UnixListener = !;

#[cfg(feature = "runtime")]
use tokio::{io, net::TcpListener};

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
    #[cfg(feature = "runtime")]
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

/// Describes a medium through which two Hydroflow services can communicate.
#[derive(Serialize, Deserialize, Debug)]
pub enum ConnectionDefn {
    UnixSocket(PathBuf),
    TcpPort(SocketAddr),
    Demux(HashMap<u32, ConnectionDefn>),
    #[cfg(feature = "runtime")]
    #[serde(skip)]
    Bind(BoundConnection),
}

#[cfg(feature = "runtime")]
impl ConnectionDefn {
    pub async fn connect<T: Connected>(self) -> T {
        T::from_defn(self).await
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

    async fn from_defn(pipe: ConnectionDefn) -> Self;
}

#[cfg(feature = "runtime")]
#[derive(Debug)]
pub enum BoundConnection {
    UnixSocket(UnixListener, tempfile::TempDir),
    TcpPort(TcpListener),
    Demux(HashMap<u32, BoundConnection>),
}

#[cfg(feature = "runtime")]
impl BoundConnection {
    pub fn connection_defn(&self) -> ConnectionDefn {
        match self {
            BoundConnection::UnixSocket(listener, _) => {
                #[cfg(unix)]
                {
                    ConnectionDefn::UnixSocket(
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
                ConnectionDefn::TcpPort(SocketAddr::new(addr.ip(), addr.port()))
            }

            BoundConnection::Demux(bindings) => {
                let mut demux = HashMap::new();
                for (key, bind) in bindings {
                    demux.insert(*key, bind.connection_defn());
                }
                ConnectionDefn::Demux(demux)
            }
        }
    }
}
