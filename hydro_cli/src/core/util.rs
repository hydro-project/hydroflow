use std::time::Duration;

use anyhow::Result;
use futures::Future;

pub async fn async_retry<T, F: Future<Output = Result<T>>>(
    thunk: impl Fn() -> F,
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
