use std::any::Any;
use std::net::SocketAddr;
use std::task::{Context, Poll};

use serde::de::DeserializeOwned;

use crate::collections::{Single};
use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;
use crate::lattice::map_union::{MapUnionRepr};
use crate::metadata::Order;
use crate::tag;
use crate::tcp_server::TcpServer;
use crate::tcp_server::serde::deserialize;

use super::optrait::*;

pub struct TcpServerOp<Lr: Any + LatticeRepr>
where
    Lr::Repr: DeserializeOwned,
{
    tcp_server: TcpServer,
    msgs: std::cell::Cell<usize>,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: Any + LatticeRepr> TcpServerOp<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    pub fn new(tcp_server: TcpServer) -> Self {
        Self {
            tcp_server,
            msgs: Default::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: Any + LatticeRepr> Op for TcpServerOp<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    type LatRepr = MapUnionRepr<tag::SINGLE, SocketAddr, Lr>;

    fn propegate_saturation(&self) {
        unimplemented!("TODO");
    }
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl<Lr: Any + LatticeRepr> OpDelta for TcpServerOp<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {

        match self.tcp_server.poll_accept(ctx) {
            Poll::Ready(Ok(addr)) => println!("New client! {}", addr),
            Poll::Ready(Err(err)) => eprintln!("Accept err! {}", err),
            Poll::Pending => (),
        }

        match self.tcp_server.poll_read(ctx) {
            Poll::Ready(Some((addr, bytes_mut))) => {
                match deserialize::<Lr>(&*bytes_mut) {
                    Ok(repr) => {
                        {
                            let msgs = self.msgs.get() + 1;
                            if 1 == msgs || 0 == msgs % 20000 {
                                let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                                println!("{} MESSAGES RECIEVED: {}", time, msgs);
                            }
                            self.msgs.set(msgs);
                        }
                        Poll::Ready(Some(Hide::new(Single((addr, repr)))))
                    }
                    Err(err) => {
                        eprintln!("Failed to deserialize: {}", err);
                        Poll::Pending
                    }
                }
            }
            _ => Poll::Pending,
        }
    }
}
