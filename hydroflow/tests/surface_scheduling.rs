use std::error::Error;

use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax, rassert_eq};
use multiplatform_test::multiplatform_test;

#[multiplatform_test(test, wasm, env_tracing)]
pub fn test_stratum_loop() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([0]) -> union_tee;
        union_tee = union() -> tee();
        union_tee -> map(|n| n + 1) -> filter(|&n| n < 10) -> next_stratum() -> union_tee;
        union_tee -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_graphvis_snapshots!(df);
    df.run_available();

    assert_eq!(
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut out_recv)
    );
    assert_eq!((11, 0), (df.current_tick(), df.current_stratum()));
}

#[multiplatform_test(test, wasm, env_tracing)]
pub fn test_tick_loop() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([0]) -> union_tee;
        union_tee = union() -> tee();
        union_tee -> map(|n| n + 1) -> filter(|&n| n < 10) -> defer_tick() -> union_tee;
        union_tee -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_graphvis_snapshots!(df);
    df.run_available();

    assert_eq!(
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut out_recv)
    );
    assert_eq!((10, 0), (df.current_tick(), df.current_stratum()));
}

#[multiplatform_test(hydroflow, env_tracing)]
async fn test_persist_stratum_run_available() -> Result<(), Box<dyn Error>> {
    let (out_send, out_recv) = hydroflow::util::unbounded_channel();

    let handle = tokio::task::spawn_blocking(|| {
        let mut df = hydroflow_syntax! {
            a = source_iter([0])
                -> persist()
                -> next_stratum()
                -> for_each(|x| out_send.send(x).unwrap());
        };
        assert_graphvis_snapshots!(df);
        df.run_available();
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    handle.abort();

    let seen: Vec<_> = hydroflow::util::collect_ready_async(out_recv).await;
    rassert_eq!(
        &[0],
        &*seen,
        "Only one tick should have run, actually ran {}",
        seen.len()
    )?;

    handle.await.unwrap();
    Ok(())
}

#[multiplatform_test(hydroflow, env_tracing)]
async fn test_persist_stratum_run_async() -> Result<(), Box<dyn Error>> {
    let (out_send, out_recv) = hydroflow::util::unbounded_channel();

    let mut df = hydroflow_syntax! {
        source_iter([0])
            -> persist()
            -> next_stratum()
            -> for_each(|x| out_send.send(x).unwrap());
    };
    assert_graphvis_snapshots!(df);

    tokio::time::timeout(std::time::Duration::from_millis(200), df.run_async())
        .await
        .expect_err("Expected time out");

    let seen: Vec<_> = hydroflow::util::collect_ready_async(out_recv).await;
    rassert_eq!(
        &[0],
        &*seen,
        "Only one tick should have run, actually ran {}",
        seen.len()
    )?;

    Ok(())
}
