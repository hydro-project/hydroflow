use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        merge();
    };
    df.run_available();
}
