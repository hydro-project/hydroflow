use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use async_channel::Sender;
use futures::future::join_all;
use futures::{Future, StreamExt};
use futures_core::Stream;

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
    Arc<Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
    Arc<Mutex<Vec<Sender<String>>>>,
);

pub fn prioritized_broadcast<T: Stream<Item = io::Result<String>> + Send + Unpin + 'static>(
    mut lines: T,
    default: impl Fn(String) + Send + 'static,
) -> PriorityBroadcacst {
    let priority_receivers = Arc::new(Mutex::new(None::<tokio::sync::oneshot::Sender<String>>));
    let receivers = Arc::new(Mutex::new(Vec::<Sender<String>>::new()));

    let weak_priority_receivers = Arc::downgrade(&priority_receivers);
    let weak_receivers = Arc::downgrade(&receivers);

    tokio::spawn(async move {
        while let Some(Result::Ok(line)) = lines.next().await {
            if let Some(cli_receivers) = weak_priority_receivers.upgrade() {
                let mut cli_receivers = cli_receivers.lock().unwrap();

                let successful_send = if let Some(r) = cli_receivers.take() {
                    r.send(line.clone()).is_ok()
                } else {
                    false
                };
                drop(cli_receivers);

                if successful_send {
                    continue;
                }
            }

            if let Some(receivers) = weak_receivers.upgrade() {
                let send_all = {
                    let mut receivers = receivers.lock().unwrap();
                    receivers.retain(|receiver| !receiver.is_closed());
                    join_all(receivers.iter().map(|receiver| {
                        // Create a future which doesn't need to hold the `receivers` lock.
                        let receiver = receiver.clone();
                        let line = &line;
                        async move { receiver.send(line.clone()).await }
                    }))
                    // Do not `.await` while holding onto the `std::sync::Mutex` `receivers` lock.
                };

                let successful_send = send_all.await.into_iter().any(|result| result.is_ok());
                if !successful_send {
                    (default)(line);
                }
            } else {
                break;
            }
        }

        if let Some(cli_receivers) = weak_priority_receivers.upgrade() {
            let mut cli_receivers = cli_receivers.lock().unwrap();
            drop(cli_receivers.take());
        }

        if let Some(receivers) = weak_receivers.upgrade() {
            let mut receivers = receivers.lock().unwrap();
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

        receivers.lock().unwrap().push(tx2);

        tx.send(Ok("hello".to_string())).await.unwrap();
        assert_eq!(rx2.next().await, Some("hello".to_string()));

        let wait_again = tokio::spawn(async move { rx2.next().await });

        drop(tx);

        assert_eq!(wait_again.await.unwrap(), None);
    }
}
