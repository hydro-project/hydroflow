use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        m = merge() -> for_each(std::mem::drop);
        source_iter(0..10) -> m;
        source_iter("hello".chars()) -> m;
    };
    df.run_available();
}
