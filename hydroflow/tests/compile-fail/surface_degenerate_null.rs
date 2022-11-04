use hydroflow::hydroflow_syntax;

/// I suppose technically this should/could compile, but it is a completely useless edge case.
fn main() {
    let mut df = hydroflow_syntax! {
        null();
    };
    df.run_available();
}
