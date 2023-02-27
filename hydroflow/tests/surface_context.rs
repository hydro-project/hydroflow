use multiplatform_test::multiplatform_test;

use hydroflow::hydroflow_syntax;

#[multiplatform_test]
pub fn test_context_ref() {
    let mut df = hydroflow_syntax! {
        source_iter([()])
            -> for_each(|()| println!("Current tick: {}, stratum: {}", context.current_tick(), context.current_stratum()));
    };
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
    df.run_available();
}
