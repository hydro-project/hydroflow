use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        src = source_iter(0..10);
        [0]src -> for_each(drop);
    };
    df.run_available();
}
