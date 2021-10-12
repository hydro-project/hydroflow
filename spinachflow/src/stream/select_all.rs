use std::pin::{Pin};
use std::task::{Context, Poll};

use futures::stream::Stream;
use pin_project::pin_project;


#[pin_project]
pub struct SelectArr<S: Stream + Unpin, const N: usize> {
    #[pin]
    streams: [S; N],
    i: usize,
}
impl<S: Stream + Unpin, const N: usize> SelectArr<S, N> {
    pub fn new(streams: [S; N]) -> Self {
        Self {
            streams,
            i: 0,
        }
    }
}
impl<S: Stream + Unpin, const N: usize> Stream for SelectArr<S, N> {
    type Item = S::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let mut not_ready = Poll::Ready(None);
        for _ in 0..N {
            let stream = Pin::new(&mut this.streams[*this.i]);
            not_ready = match stream.poll_next(cx) {
                Poll::Ready(Some(delta)) => return Poll::Ready(Some(delta)),
                Poll::Ready(None) => not_ready,
                Poll::Pending => Poll::Pending,
            };
            *this.i = (*this.i + 1) % N;
        }
        // let t = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        // match not_ready {
        //     Poll::Ready(Some(_)) => println!("{}: Poll::Ready(Some(_))", t),
        //     Poll::Ready(None) => println!("{}: Poll::Ready(None)", t),
        //     Poll::Pending => println!("{}: Poll::Pending", t),
        // }
        not_ready
    }
}

#[tokio::test]
async fn test_select_arr() {
    const NUM_OPS: usize = 5;
    const NUM_INTS: usize = 1000;

    use crate::futures::StreamExt;
    use crate::futures::future::ready;

    let streams = [(); NUM_OPS].map(|_| {
        crate::futures::stream::iter(0..NUM_INTS)
    });
    let stream = SelectArr::new(streams);
    let stream = stream.map(|x| ready(x));
    let mut stream = stream;
    while stream.next().await.is_some() {}
}
