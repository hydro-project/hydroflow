use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        f = f -> f;
    };
    df.run_available();
}
