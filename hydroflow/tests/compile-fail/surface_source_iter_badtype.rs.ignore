use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        source_iter(()) -> for_each(std::mem::drop);
    };
    df.run_available();
}
