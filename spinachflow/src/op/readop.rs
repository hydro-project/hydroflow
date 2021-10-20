use std::cell::RefCell;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader, Lines, Stdin};

use crate::collections::Single;
use crate::hide::{Delta, Hide};
use crate::lattice::set_union::SetUnionRepr;
use crate::metadata::Order;
use crate::tag::SINGLE;

use super::*;

pub struct ReadOp<R: AsyncRead + Unpin> {
    reader: RefCell<Lines<BufReader<R>>>,
}

impl ReadOp<Stdin> {
    pub fn new_stdin() -> Self {
        Self {
            reader: RefCell::new(BufReader::new(tokio::io::stdin()).lines()),
        }
    }
}

impl<R: AsyncRead + Unpin> ReadOp<R> {
    pub fn new(read: R) -> Self {
        Self {
            reader: RefCell::new(BufReader::new(read).lines()),
        }
    }

    pub fn from_buf(buf_read: BufReader<R>) -> Self {
        Self {
            reader: RefCell::new(buf_read.lines()),
        }
    }
}

impl<R: AsyncRead + Unpin> Op for ReadOp<R> {
    type LatRepr = SetUnionRepr<SINGLE, String>;

    fn propegate_saturation(&self) {
        unimplemented!("TODO?");
    }
}

impl<R: AsyncRead + Unpin> OpDelta for ReadOp<R> {
    type Ord = UserInputOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        loop {
            match Pin::new(&mut *self.reader.borrow_mut())
                .as_mut()
                .poll_next_line(ctx)
            {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Result::Ok(opt)) => {
                    return Poll::Ready(opt.map(|x| Hide::new(Single(x))))
                }
                Poll::Ready(Result::Err(err)) => println!("ERROR: {}", err),
            }
        }
    }
}

pub struct UserInputOrder;
impl Order for UserInputOrder {}
