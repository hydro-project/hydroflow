use hydroflow::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
        null();
    };
    df.run_available();
}
