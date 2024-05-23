use std::collections::HashSet;

use hydroflow::hydroflow_syntax;
use hydroflow::util::collect_ready_async;
use tokio::time::{sleep, Duration};

#[hydroflow::test]
async fn single_batch_test() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u32>();

    let mut df = hydroflow_syntax! {
        source_iter(0..10)
        -> map(|x| async move {
            sleep(Duration::from_millis(100)).await;
            x
        })
        -> poll_futures()
        -> for_each(|x| result_send.send(x).unwrap());
    };

    let handle = tokio::task::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        assert_eq!(
            HashSet::from_iter([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            collect_ready_async::<HashSet<_>, _>(&mut result_recv).await
        );
    });

    tokio::time::timeout(Duration::from_secs(2), df.run_async())
        .await
        .expect_err("Expected time out");

    handle.await.unwrap();
}

#[hydroflow::test]
async fn multi_batch_test() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u64>();

    let mut df = hydroflow_syntax! {
        source_iter([2, 3, 1, 9, 6, 5, 4, 7, 8])
        -> map(|x| async move {
            sleep(Duration::from_millis(10*x)).await;
            x
        })
        -> poll_futures()
        -> for_each(|x| result_send.send(x).unwrap());
    };

    let handle = tokio::task::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        assert_eq!(
            HashSet::from_iter([1, 2, 3, 4, 5, 6, 7, 8, 9]),
            collect_ready_async::<HashSet<_>, _>(&mut result_recv).await
        );
    });

    tokio::time::timeout(Duration::from_secs(2), df.run_async())
        .await
        .expect_err("Expected time out");

    handle.await.unwrap();
}

#[hydroflow::test]
async fn pusherator_test() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u64>();

    let mut df = hydroflow_syntax! {
        ins = source_iter([2, 3, 1, 9, 6, 5, 4, 7, 8])
            -> tee();

        ins -> for_each(|_| {});
        ins -> map(|x| async move {
            sleep(Duration::from_millis(10*x)).await;
            x
        }) -> poll_futures() -> for_each(|x| result_send.send(x).unwrap());
    };

    let handle = tokio::task::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        assert_eq!(
            HashSet::from_iter([1, 2, 3, 4, 5, 6, 7, 8, 9]),
            collect_ready_async::<HashSet<_>, _>(&mut result_recv).await
        );
    });

    tokio::time::timeout(Duration::from_secs(2), df.run_async())
        .await
        .expect_err("Expected time out");

    handle.await.unwrap();
}
