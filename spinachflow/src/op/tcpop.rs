use std::any::Any;
use std::cell::RefCell;
use std::task::{Context, Poll};
use std::pin::Pin;

use futures_core::stream::Stream;
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use serde::de::DeserializeOwned;

use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;
use crate::metadata::Order;
use crate::tcp_server::serde::deserialize;

use super::optrait::*;

pub struct TcpOp<Lr: Any + LatticeRepr>
where
    Lr::Repr: DeserializeOwned,
{
    framed_read: RefCell<FramedRead<OwnedReadHalf, LengthDelimitedCodec>>,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: Any + LatticeRepr> TcpOp<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    pub fn new(tcp_read: OwnedReadHalf) -> Self {
        let framed_read = LengthDelimitedCodec::builder()
            .length_field_length(2)
            .new_read(tcp_read);
        Self {
            framed_read: RefCell::new(framed_read),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: Any + LatticeRepr> Op for TcpOp<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
        unimplemented!("TODO");
    }
}

pub enum TcpOrder {}
impl Order for TcpOrder {}

impl<Lr: Any + LatticeRepr> OpDelta for TcpOp<Lr>
where
    Lr::Repr: DeserializeOwned,
{
    type Ord = TcpOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match Pin::new(&mut *self.framed_read.borrow_mut()).poll_next(ctx) {
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                println!("TcpOp Error {}", err);
                Poll::Pending
            }
            Poll::Ready(Some(Ok(bytes_mut))) => {
                match deserialize::<Lr>(&*bytes_mut) {
                    Ok(repr) => Poll::Ready(Some(Hide::new(repr))),
                    Err(err) => {
                        eprintln!("Failed to deserialize: {}", err);
                        Poll::Pending
                    }
                }
            }
            Poll::Pending => Poll::Pending
        }
    }
}
