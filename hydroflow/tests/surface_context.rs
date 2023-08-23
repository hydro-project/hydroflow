use std::cell::Cell;
use std::rc::Rc;

use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use instant::{Duration, Instant};
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_context_ref() {
    let mut df = hydroflow_syntax! {
        source_iter([()])
            -> for_each(|()| println!("Current tick: {}, stratum: {}", context.current_tick(), context.current_stratum()));
    };
    assert_graphvis_snapshots!(df);
    df.run_available();
}

#[multiplatform_test]
pub fn test_context_mut() {
    // TODO(mingwei): Currently cannot have conflicting (mut) references to `context` in the same
    // subgraph - bit of a leak of the subgraphs abstraction. `next_stratum()` here so it runs.
    let mut df = hydroflow_syntax! {
        source_iter(0..10)
            -> map(|n| context.add_state(n))
            -> next_stratum()
            -> for_each(|handle| println!("{:?}: {}", handle, context.state_ref(handle)));
    };
    assert_graphvis_snapshots!(df);
    df.run_available();
}

#[multiplatform_test]
pub fn test_context_current_tick_start() {
    let mut df = hydroflow_syntax! {
        source_iter([()])
            -> map(|_| context.current_tick_start())
            -> defer_tick()
            -> assert(|t: &hydroflow::instant::Instant| t.elapsed().as_nanos() > 0)
            -> for_each(|t: hydroflow::instant::Instant| println!("Time between ticks: {:?}", t.elapsed()));
    };
    assert_graphvis_snapshots!(df);
    df.run_available();
}

#[multiplatform_test(hydroflow)]
pub async fn test_context_current_tick_start_does_not_count_time_between_ticks_async() {
    let time = Rc::new(Cell::new(None));

    let mut df = {
        let time = time.clone();
        hydroflow_syntax! {
            source_iter([()])
                -> persist()
                -> for_each(|_| time.set(Some(Instant::now() - context.current_tick_start())));
        }
    };
    assert_graphvis_snapshots!(df);
    tokio::time::sleep(Duration::from_millis(100)).await;
    df.run_tick();
    assert!(time.take().unwrap() < Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(100)).await;
    df.run_tick();
    assert!(time.take().unwrap() < Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(100)).await;
    df.run_tick_async().await;
    assert!(time.take().unwrap() < Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(100)).await;
    df.run_available();
    assert!(time.take().unwrap() < Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(100)).await;
    df.run_available_async().await;
    assert!(time.take().unwrap() < Duration::from_millis(50));
}
