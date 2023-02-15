use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        x = for_each(std::mem::drop);
        source_iter(0..10) [0]x;
    };
    df.run_available();
}
