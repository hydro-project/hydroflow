use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        null();
    };
    df.run_available();
}
