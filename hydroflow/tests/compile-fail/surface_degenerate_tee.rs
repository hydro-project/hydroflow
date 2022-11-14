use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        tee();
    };
    df.run_available();
}
