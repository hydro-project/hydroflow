use std::any::Any;
use std::cell::RefCell;
use std::future::Future;

use bincode::{Error, ErrorKind};
use futures::sink::SinkExt;
use serde::ser::Serialize;
use tokio::net::tcp::OwnedWriteHalf;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use crate::lattice::LatticeRepr;
use crate::op::OpDelta;
use crate::tcp_server::serde::serialize;

use super::{Comp, Next};

pub struct TcpComp<O: OpDelta>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    op: O,
    framed_write: RefCell<FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>>,
}

impl<O: OpDelta> TcpComp<O>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    pub fn new(op: O, tcp_write: OwnedWriteHalf) -> Self {
        let framed_write = LengthDelimitedCodec::builder()
            .length_field_length(2)
            .new_write(tcp_write);
        Self {
            op,
            framed_write: RefCell::new(framed_write),
        }
    }
}

impl<O: OpDelta> Comp for TcpComp<O>
where
    O::LatRepr: Any,
    <O::LatRepr as LatticeRepr>::Repr: Serialize,
{
    type Error = Error;

    type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
    fn tick(&self) -> Self::TickFuture<'_> {
        async move {
            let mut framed_write_mut = self.framed_write.borrow_mut();
            if let Some(hide) = (Next { op: &self.op }).await {
                let bytes = serialize::<O::LatRepr>(hide.into_reveal())?.freeze();
                framed_write_mut.send(bytes).await?;
                Ok(())
            }
            else {
                // framed_write_mut.shutdown().await?;
                Err(Box::new(ErrorKind::Custom("End of stream.".to_owned())))
            }
        }
    }
}
