use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        src = source_iter(0..10);
        dst = for_each(drop);
        [0]src -> dst[0];
    };
    df.run_available();
}
