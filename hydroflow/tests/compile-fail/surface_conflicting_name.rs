use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        a = null() -> null();
        a = null() -> null();
    };
    df.run_available();
}
