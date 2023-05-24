use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        union();
    };
    df.run_available();
}
