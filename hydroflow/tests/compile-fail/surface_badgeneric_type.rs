use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        // TODO(mingwei): this may make sense at some point in the future.
        // Currently no generic arguments implemented for `identity` (or `map`, etc.).
        source_iter(0..10) -> identity::<usize>() -> for_each(std::mem::drop);
    };
    df.run_available();
}
