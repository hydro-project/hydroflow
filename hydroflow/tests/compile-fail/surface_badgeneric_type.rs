use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        // no generic arguments for `next_tick`.
        source_iter(0..10) -> next_tick::<usize>() -> for_each(std::mem::drop);
    };
    df.run_available();
}
