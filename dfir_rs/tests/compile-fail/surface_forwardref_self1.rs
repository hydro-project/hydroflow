use dfir_rs::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
        f = f -> f;
    };
    df.run_available();
}
