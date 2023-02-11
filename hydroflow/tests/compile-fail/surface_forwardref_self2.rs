use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        f = f -> map(|x: usize| x) -> f;
    };
    df.run_available();
}
