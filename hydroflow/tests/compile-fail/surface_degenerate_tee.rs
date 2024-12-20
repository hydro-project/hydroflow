use hydroflow::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
        tee();
    };
    df.run_available();
}
