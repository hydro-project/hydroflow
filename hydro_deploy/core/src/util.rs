use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_channel::Sender;
use futures::{Future, StreamExt};
use futures_core::Stream;
use tokio::sync::RwLock;

pub async fn async_retry<T, F: Future<Output = Result<T>>>(
    mut thunk: impl FnMut() -> F,
    count: usize,
    delay: Duration,
) -> Result<T> {
    for _ in 1..count {
        let result = thunk().await;
        if result.is_ok() {
            return result;
        } else {
            tokio::time::sleep(delay).await;
        }
    }

    thunk().await
}

type PriorityBroadcacst = (
    Arc<RwLock<Option<tokio::sync::oneshot::Sender<String>>>>,
    Arc<RwLock<Vec<Sender<String>>>>,
);

// Divides up a single stream into two channels (one prioritized and the other not)
// The first data packet that comes in will get sent to the first channel, and all others will get sent to the second channel
pub fn prioritized_broadcast<T: Stream<Item = io::Result<String>> + Send + Unpin + 'static>(
    mut lines: T,
    default: impl Fn(String) + Send + 'static,
) -> PriorityBroadcacst {
    let priority_receivers = Arc::new(RwLock::new(None::<tokio::sync::oneshot::Sender<String>>));
    let receivers = Arc::new(RwLock::new(Vec::<Sender<String>>::new()));

    let weak_priority_receivers = Arc::downgrade(&priority_receivers);
    let weak_receivers = Arc::downgrade(&receivers);

    tokio::spawn(async move {
        'line_loop: while let Some(Result::Ok(line)) = lines.next().await {
            if let Some(cli_receivers) = weak_priority_receivers.upgrade() {
                let mut cli_receivers = cli_receivers.write().await;

                let successful_send = if let Some(r) = cli_receivers.take() {
                    r.send(line.clone()).is_ok()
                } else {
                    false
                };

                if successful_send {
                    continue 'line_loop;
                }
            }

            if let Some(receivers) = weak_receivers.upgrade() {
                let mut receivers = receivers.write().await;
                let mut successful_send = false;
                for r in receivers.iter() {
                    successful_send |= r.send(line.clone()).await.is_ok();
                }

                receivers.retain(|r| !r.is_closed());

                if !successful_send {
                    default(line);
                }
            } else {
                break;
            }
        }

        if let Some(cli_receivers) = weak_priority_receivers.upgrade() {
            let mut cli_receivers = cli_receivers.write().await;
            drop(cli_receivers.take());
        }

        if let Some(receivers) = weak_receivers.upgrade() {
            let mut receivers = receivers.write().await;
            receivers.clear();
        }
    });

    (priority_receivers, receivers)
}

#[cfg(test)]
mod test {
    use futures::StreamExt;

    #[tokio::test]
    async fn broadcast_listeners_close_when_source_does() {
        let (tx, rx) = async_channel::unbounded::<_>();
        let (_, receivers) = super::prioritized_broadcast(rx, |_| {});

        let (tx2, mut rx2) = async_channel::unbounded::<_>();

        receivers.try_write().unwrap().push(tx2);

        tx.send(Ok("hello".to_string())).await.unwrap();
        assert_eq!(rx2.next().await, Some("hello".to_string()));

        let wait_again = tokio::spawn(async move { rx2.next().await });

        drop(tx);

        assert_eq!(wait_again.await.unwrap(), None);
    }
}
