use hydroflow::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
        union();
    };
    df.run_available();
}
