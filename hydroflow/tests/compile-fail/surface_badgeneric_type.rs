use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        // no generic arguments for `defer`.
        source_iter(0..10) -> defer_tick::<usize>() -> for_each(std::mem::drop);
    };
    df.run_available();
}
