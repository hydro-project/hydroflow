use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        g = f -> map(|x: usize| x) -> f;
        f = g;
    };
    df.run_available();
}
